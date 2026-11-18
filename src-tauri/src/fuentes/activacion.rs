//! Las carpetas del catálogo son de solo lectura. Solo registramos copias propias.
use std::{fs, path::{Path, PathBuf}};

use super::{cambios::libre, clave_ruta, leer::Cara, registro, Entorno};
use crate::win;

pub fn activar(ent: &Entorno, original: &Path, caras: &[Cara]) -> Result<(), String> {
    if caras.is_empty() { return Err("No font faces".into()); }
    let nombres = caras.iter().map(|c| c.completo.clone()).collect::<Vec<_>>();
    let cff = original.extension().is_some_and(|e| e.eq_ignore_ascii_case("otf"));
    let valor = registro::nombre_valor(&nombres, cff);
    for (raiz, clave) in [(registro::hkcu(), &ent.clave_usuario), (registro::hklm(), &ent.clave_sistema)] {
        if let Some((_, datos)) = registro::leer(raiz, clave).into_iter().find(|(n, _)| n.eq_ignore_ascii_case(&valor)) {
            if raiz == registro::hkcu() && Path::new(&datos).parent() == Some(ent.activas().as_path()) { return Ok(()); }
            return Err(format!("Already registered: {valor}"));
        }
    }

    let dir = ent.activas();
    fs::create_dir_all(&dir).map_err(crate::err)?;
    let nombre = original.file_name().ok_or("Missing filename")?.to_string_lossy();
    let copia = libre(&dir, &nombre);
    if let Err(e) = fs::copy(original, &copia) {
        let _ = fs::remove_file(&copia);
        return Err(e.to_string());
    }
    if ent.tocar_windows && !win::cargar(&copia, false) {
        let _ = fs::remove_file(&copia);
        return Err(format!("Windows could not load {nombre}"));
    }
    if let Err(e) = registro::poner(registro::hkcu(), &ent.clave_usuario, &valor, &copia.to_string_lossy()) {
        if ent.tocar_windows { win::descargar(&copia, false); }
        let _ = fs::remove_file(&copia);
        return Err(e);
    }
    Ok(())
}

pub fn desactivar(ent: &Entorno, valor: &str, copia: &Path) -> Result<(), String> {
    if copia.parent() != Some(ent.activas().as_path()) { return Err("External font".into()); }
    let propia = registro::leer(registro::hkcu(), &ent.clave_usuario).iter().any(|(n, d)| {
        n == valor && clave_ruta(Path::new(d)) == clave_ruta(copia)
    });
    if !propia { return Ok(()); }
    registro::quitar(registro::hkcu(), &ent.clave_usuario, valor)?;
    if ent.tocar_windows { win::descargar(copia, false); }
    // Un editor puede mantener el archivo abierto. Ya no está registrado.
    let _ = fs::remove_file(copia);
    Ok(())
}

pub fn desactivar_propias(ent: &Entorno) -> Result<(), String> {
    let mut errores = Vec::new();
    let dir = ent.activas();
    for (valor, ruta) in registro::leer(registro::hkcu(), &ent.clave_usuario) {
        let ruta = PathBuf::from(ruta);
        if ruta.parent() == Some(dir.as_path()) {
            if let Err(e) = desactivar(ent, &valor, &ruta) { errores.push(e); }
        }
    }
    if ent.tocar_windows { win::avisar_cambio(); }
    if errores.is_empty() { Ok(()) } else { Err(errores.join("\n")) }
}
