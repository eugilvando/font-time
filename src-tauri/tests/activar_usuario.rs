#![cfg(windows)]
//! Activar y desactivar una fuente de usuario de prueba, sin tocar las de
//! verdad: carpetas temporales y una clave de registro propia en HKCU que se
//! borra al acabar. No se carga nada en Windows (tocar_windows = false).

use std::{collections::HashSet, path::Path};
use tunafonts_lib::fuentes::{self, cache::Cache, cambios, leer, registro, Entorno, Origen};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

struct Limpieza(String);
impl Drop for Limpieza {
    fn drop(&mut self) {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let _ = hkcu.delete_subkey_all(&self.0);
        // Y la carpeta de pruebas, si se ha quedado vacía.
        let _ = hkcu.delete_subkey(r"Software\tuna-fonts-pruebas");
    }
}

#[test]
fn instalar_desactivar_y_activar_una_de_usuario() {
    let tmp = tempfile::tempdir().unwrap();
    let base = format!(r"Software\tuna-fonts-pruebas\{}", std::process::id());
    let _limpia = Limpieza(base.clone());
    let ent = Entorno {
        fuentes_sistema: tmp.path().join("sistema"),
        fuentes_usuario: tmp.path().join("usuario"),
        clave_sistema: format!(r"{base}\Sistema"),
        clave_usuario: format!(r"{base}\Fonts"),
        datos: tmp.path().join("datos"),
        tocar_windows: false,
        de_windows: |_| true,
        de_programas: HashSet::new(),
    };
    let fuente = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/GochiHand-Regular.ttf");

    // Leer: familia y categoría.
    let caras = leer::caras(&fuente).unwrap();
    assert_eq!(caras[0].familia, "Gochi Hand");
    assert_eq!(caras[0].categoria, "script");
    assert!(!leer::caracteres(&fuente, 500).unwrap().is_empty());

    // Instalar para el usuario.
    let instalada = cambios::instalar_usuario(&ent, &fuente, &caras).unwrap();
    assert!(instalada.is_file());
    let reg = registro::leer(registro::hkcu(), &ent.clave_usuario);
    assert_eq!(reg.len(), 1);
    assert_eq!(reg[0].0, "Gochi Hand (TrueType)");
    assert_eq!(Path::new(&reg[0].1), instalada);

    let mut cache = Cache::default();
    let entradas = fuentes::entradas(&ent, &[]);
    cache.actualizar(&entradas);
    let f = cache.familias.iter().find(|f| f.nombre == "Gochi Hand").unwrap();
    assert!(f.activa);
    assert_eq!(f.origen, Origen::Biblioteca);

    // Desactivar: fuera del registro y a la carpeta de la app, sin borrar.
    let r = cambios::alternar(&ent, &entradas, &cache, &["Gochi Hand".into()], false, false);
    assert_eq!(r.cambiadas, 1, "{:?}", r.errores);
    assert!(!instalada.exists());
    assert!(registro::leer(registro::hkcu(), &ent.clave_usuario).is_empty());
    let g = fuentes::guardadas(&ent);
    assert_eq!(g.len(), 1);
    assert!(g[0].archivo.starts_with(ent.desactivadas()) && g[0].archivo.is_file());

    let entradas = fuentes::entradas(&ent, &[]);
    cache.actualizar(&entradas);
    let f = cache.familias.iter().find(|f| f.nombre == "Gochi Hand").unwrap();
    assert!(!f.activa);

    // Desactivar otra vez no hace nada.
    let r = cambios::alternar(&ent, &entradas, &cache, &["Gochi Hand".into()], false, false);
    assert_eq!(r.cambiadas, 0);

    // Activar: vuelve a su sitio y a su entrada del registro.
    let r = cambios::alternar(&ent, &entradas, &cache, &["Gochi Hand".into()], true, false);
    assert_eq!(r.cambiadas, 1, "{:?}", r.errores);
    assert!(instalada.is_file());
    let reg = registro::leer(registro::hkcu(), &ent.clave_usuario);
    assert_eq!(reg, vec![("Gochi Hand (TrueType)".to_string(), instalada.to_string_lossy().into_owned())]);
    assert!(fuentes::guardadas(&ent).is_empty());

    let entradas = fuentes::entradas(&ent, &[]);
    cache.actualizar(&entradas);
    assert!(cache.familias.iter().find(|f| f.nombre == "Gochi Hand").unwrap().activa);
}

#[test]
fn las_imprescindibles_no_se_tocan() {
    for f in ["Segoe UI", "Segoe UI Variable", "Segoe UI Emoji", "Segoe Fluent Icons", "Segoe MDL2 Assets", "Marlett", "Tahoma", "Microsoft Sans Serif", "Arial", "Consolas"] {
        assert!(fuentes::es_imprescindible(f), "{f}");
    }
    for f in ["Arial Narrow", "Georgia", "Times New Roman", "Segoe Print"] {
        assert!(!fuentes::es_imprescindible(f), "{f}");
    }
}

/// En la carpeta de fuentes del sistema, solo las que trae Windows van con
/// candado; las instaladas a mano «para todos» (u Office) salen como de usuario.
#[test]
fn las_instaladas_para_todos_no_cuentan_como_del_sistema() {
    let tmp = tempfile::tempdir().unwrap();
    let ent = Entorno {
        fuentes_sistema: tmp.path().join("sistema"),
        fuentes_usuario: tmp.path().join("usuario"),
        clave_sistema: r"Software\tuna-fonts-pruebas\no-existe".into(),
        clave_usuario: r"Software\tuna-fonts-pruebas\no-existe".into(),
        datos: tmp.path().join("datos"),
        tocar_windows: false,
        // Simula el propietario: solo «de-windows.ttf» es de TrustedInstaller.
        de_windows: |p| p.file_name().is_some_and(|n| n == "de-windows.ttf"),
        // Y «de-office.ttf» la trae Office.
        de_programas: ["de-office.ttf".to_string()].into(),
    };
    let fuente = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/GochiHand-Regular.ttf");
    std::fs::create_dir_all(&ent.fuentes_sistema).unwrap();
    std::fs::copy(&fuente, ent.fuentes_sistema.join("de-windows.ttf")).unwrap();
    std::fs::copy(&fuente, ent.fuentes_sistema.join("a-mano.ttf")).unwrap();
    std::fs::copy(&fuente, ent.fuentes_sistema.join("DE-OFFICE.TTF")).unwrap();

    let origen = |nombre: &str| {
        fuentes::entradas(&ent, &[]).into_iter().find(|e| e.ruta.file_name().unwrap() == nombre).unwrap().origen
    };
    assert_eq!(origen("de-windows.ttf"), Origen::Sistema);
    assert_eq!(origen("a-mano.ttf"), Origen::Usuario);
    assert_eq!(origen("DE-OFFICE.TTF"), Origen::Sistema);

    // Una de Windows que la app ya desactivó y volvió a activar (ahora es una
    // copia, con otro propietario) sigue siendo del sistema.
    let mut apuntadas = HashSet::new();
    apuntadas.insert(fuentes::clave_ruta(&ent.fuentes_sistema.join("a-mano.ttf")));
    fuentes::poner_de_windows_apuntadas(&ent, &apuntadas).unwrap();
    assert_eq!(origen("a-mano.ttf"), Origen::Sistema);
}

/// En un Windows de verdad, Arial es de TrustedInstaller.
#[test]
fn arial_es_de_trustedinstaller() {
    let windir = std::env::var_os("WINDIR").unwrap();
    let arial = Path::new(&windir).join("Fonts").join("arial.ttf");
    assert_eq!(tunafonts_lib::win::de_trustedinstaller(&arial), Some(true));
}

#[test]
fn catalogar_activar_persistir_y_apagar_solo_las_propias() {
    let tmp = tempfile::tempdir().unwrap();
    let base = format!(r"Software\tuna-fonts-pruebas\catalogo-{}", std::process::id());
    let _limpia = Limpieza(base.clone());
    let ent = Entorno {
        fuentes_sistema: tmp.path().join("sistema"), fuentes_usuario: tmp.path().join("usuario"),
        clave_sistema: format!(r"{base}\Sistema"), clave_usuario: format!(r"{base}\Fonts"),
        datos: tmp.path().join("datos"), tocar_windows: false, de_windows: |_| false, de_programas: HashSet::new(),
    };
    let carpeta = tmp.path().join("OneDrive");
    std::fs::create_dir(&carpeta).unwrap();
    let original = carpeta.join("GochiHand.ttf");
    let muestra = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/GochiHand-Regular.ttf");
    std::fs::copy(&muestra, &original).unwrap();
    let fuentes_en = || fuentes::entradas(&ent, std::slice::from_ref(&carpeta));
    let nombre = vec!["Gochi Hand".into()];
    let mut cache = Cache::default();
    let lista = fuentes_en();
    cache.actualizar(&lista);
    assert!(!cache.familias[0].activa);
    assert!(registro::leer(registro::hkcu(), &ent.clave_usuario).is_empty());
    let r = cambios::alternar(&ent, &lista, &cache, &nombre, true, false);
    assert!(r.errores.is_empty(), "{:?}", r.errores);
    let mut reabierta = Cache::default();
    reabierta.actualizar(&fuentes_en());
    assert!(reabierta.familias[0].activa);
    assert_eq!(registro::leer(registro::hkcu(), &ent.clave_usuario).len(), 1);

    let externa = carpeta.join("externa.ttf");
    registro::poner(registro::hkcu(), &ent.clave_usuario, "otra aplicación", &externa.to_string_lossy()).unwrap();
    fuentes::activacion::desactivar_propias(&ent).unwrap();
    fuentes::activacion::desactivar_propias(&ent).unwrap();
    assert_eq!(registro::leer(registro::hkcu(), &ent.clave_usuario), vec![("otra aplicación".into(), externa.to_string_lossy().to_string())]);
    reabierta.actualizar(&fuentes_en());
    assert!(!reabierta.familias[0].activa);
    assert_eq!(std::fs::read(&original).unwrap(), std::fs::read(&muestra).unwrap());
}
