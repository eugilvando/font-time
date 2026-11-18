use std::path::{Path, PathBuf};
use serde::Serialize;

#[derive(Serialize, Default, Debug)]
pub struct Resultado {
    pub cambiadas: usize,
    /// Del sistema que no se han tocado (bloqueadas o imprescindibles).
    pub bloqueadas: usize,
    pub errores: Vec<String>,
    /// Alguna del sistema se quitará del todo al reiniciar.
    pub reiniciar: bool,
    /// Se dijo que no al aviso de permisos de Windows.
    pub cancelado: bool,
    /// Familias instaladas (para el aviso).
    pub instaladas: Vec<String>,
}

/// Una ruta libre en `dir` para `nombre`: "x.ttf", "x (2).ttf"...
pub fn libre(dir: &Path, nombre: &str) -> PathBuf {
    let p = dir.join(nombre);
    if !p.exists() {
        return p;
    }
    let base = Path::new(nombre);
    let (tallo, ext) = (
        base.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
        base.extension().map(|s| format!(".{}", s.to_string_lossy())).unwrap_or_default(),
    );
    (2..).map(|i| dir.join(format!("{tallo} ({i}){ext}"))).find(|p| !p.exists()).unwrap()
}
