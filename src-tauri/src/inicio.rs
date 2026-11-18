//! Abrir Font Time al iniciar Windows, escondida en la bandeja: así las
//! fuentes de las carpetas vigiladas están disponibles desde el principio.
//!
//! - Va en HKCU\...\Run (solo este usuario, sin permisos de administrador) con
//!   el nombre «Font Time», que es justo el valor que borra el desinstalador.
//! - Lo que manda es el registro, no un ajuste guardado: si alguien lo apaga en
//!   el Administrador de tareas (Aplicaciones de arranque) o mueve la portable,
//!   la app lo ve como apagado en vez de decir algo que no es verdad.
//! - Arranca con «--oculta». Al actualizarse, el instalador vuelve a abrir la
//!   app con los mismos argumentos; para que no reaparezca escondida (y parezca
//!   que se ha cerrado), antes de instalar se deja una marca que obliga a
//!   enseñar la ventana en el siguiente arranque.

use std::path::{Path, PathBuf};
use winreg::{
    enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE},
    RegKey,
};

use crate::err;

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Donde el Administrador de tareas apunta las que se han desactivado.
const APROBADO: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
/// El mismo nombre que borra el desinstalador de Tauri (PRODUCTNAME).
const NOMBRE: &str = "Font Time";
pub const OCULTA: &str = "--oculta";
const MARCA: &str = "mostrar-al-abrir";

fn exe() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(err)
}

fn comando(exe: &Path) -> String {
    format!("\"{}\" {OCULTA}", exe.display())
}

/// ¿Está apagada en el Administrador de tareas? El primer byte impar es «desactivada».
fn desactivada_en_windows(hkcu: &RegKey) -> bool {
    hkcu.open_subkey(APROBADO)
        .and_then(|k| k.get_raw_value(NOMBRE))
        .map(|v| v.bytes.first().is_some_and(|b| b & 1 == 1))
        .unwrap_or(false)
}

/// Arranca de verdad con Windows: está en Run, apunta a este .exe y nadie la ha
/// desactivado en el Administrador de tareas.
pub fn activo() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(v) = hkcu.open_subkey(RUN).and_then(|k| k.get_value::<String, _>(NOMBRE)) else {
        return false;
    };
    let Ok(e) = exe() else { return false };
    v.to_lowercase().contains(&e.to_string_lossy().to_lowercase()) && !desactivada_en_windows(&hkcu)
}

pub fn poner(activar: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if activar {
        let (run, _) = hkcu.create_subkey(RUN).map_err(err)?;
        run.set_value(NOMBRE, &comando(&exe()?)).map_err(err)?;
        // Si se había desactivado en el Administrador de tareas, se vuelve a permitir.
        if let Ok(k) = hkcu.open_subkey_with_flags(APROBADO, KEY_READ | KEY_WRITE) {
            let _ = k.delete_value(NOMBRE);
        }
    } else {
        if let Ok(run) = hkcu.open_subkey_with_flags(RUN, KEY_READ | KEY_WRITE) {
            if let Err(e) = run.delete_value(NOMBRE) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(err(e));
                }
            }
        }
        if let Ok(k) = hkcu.open_subkey_with_flags(APROBADO, KEY_READ | KEY_WRITE) {
            let _ = k.delete_value(NOMBRE);
        }
    }
    Ok(())
}

/// Al arrancar: ¿hay que quedarse en la bandeja? Solo con «--oculta» y sin
/// marca de «enséñate» (la deja el actualizador). La marca se gasta siempre.
pub fn arrancar_oculta(datos: &Path) -> bool {
    let marca = datos.join(MARCA);
    let habia_marca = marca.exists();
    let _ = std::fs::remove_file(&marca);
    std::env::args_os().skip(1).any(|a| a == OCULTA) && !habia_marca
}

pub fn marcar_mostrar(datos: &Path) -> Result<(), String> {
    std::fs::create_dir_all(datos).map_err(err)?;
    std::fs::write(datos.join(MARCA), b"").map_err(err)
}

#[tauri::command]
pub fn inicio_windows() -> bool {
    activo()
}

/// Devuelve cómo ha quedado de verdad (por si Windows no lo ha aceptado).
#[tauri::command]
pub fn poner_inicio_windows(activar: bool) -> Result<bool, String> {
    poner(activar)?;
    Ok(activo())
}

/// Antes de instalar una actualización: que la app vuelva a la vista al reabrirse.
#[tauri::command]
pub fn marcar_mostrar_al_abrir(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    marcar_mostrar(&app.path().app_data_dir().map_err(err)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_marca_de_mostrar_se_gasta_al_arrancar() {
        let tmp = tempfile::tempdir().unwrap();
        marcar_mostrar(tmp.path()).unwrap();
        assert!(tmp.path().join(MARCA).exists());
        // Los tests no llevan --oculta: no se esconde, y la marca desaparece igual.
        assert!(!arrancar_oculta(tmp.path()));
        assert!(!tmp.path().join(MARCA).exists());
    }

    /// Escribe de verdad en HKCU\...\Run y lo deja como estaba. A mano:
    /// `cargo test -- --ignored inicio_con_windows`.
    #[test]
    #[ignore]
    fn inicio_con_windows_se_pone_y_se_quita() {
        let antes = activo();
        poner(true).unwrap();
        assert!(activo());
        let v: String = RegKey::predef(HKEY_CURRENT_USER).open_subkey(RUN).unwrap().get_value(NOMBRE).unwrap();
        assert!(v.ends_with(OCULTA), "{v}");
        poner(false).unwrap();
        assert!(!activo());
        poner(false).unwrap(); // dos veces seguidas no falla
        if antes {
            poner(true).unwrap();
        }
    }
}
