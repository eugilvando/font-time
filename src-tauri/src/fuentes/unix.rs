use super::{cache::Cache, listar, listar_con_directorios, Entrada, Entorno, Origen};
use std::{
    collections::HashSet, fs, io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

impl Entorno {
    pub fn para_usuario(datos: PathBuf, home: &Path) -> Self {
        #[cfg(target_os = "macos")]
        let fuentes_usuario = home.join("Library/Fonts");
        #[cfg(target_os = "linux")]
        let fuentes_usuario = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from)
            .filter(|p| p.is_absolute()).unwrap_or_else(|| home.join(".local/share")).join("fonts");
        Self { datos, fuentes_usuario }
    }

    // The anchor is on the same volume as the published font. Matching both
    // inode and device lets us remove only files we actually installed.
    pub fn activas(&self) -> PathBuf {
        self.fuentes_usuario.parent().unwrap().join(".font-time-owned")
    }

    pub fn directorios_sistema(&self) -> Vec<PathBuf> {
        #[cfg(target_os = "macos")]
        { vec!["/System/Library/Fonts".into(), "/Library/Fonts".into()] }
        #[cfg(target_os = "linux")]
        {
            let dirs = std::env::var_os("XDG_DATA_DIRS").filter(|v| !v.is_empty())
                .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
            let mut fuentes: Vec<_> = std::env::split_paths(&dirs).filter(|p| p.is_absolute()).map(|p| p.join("fonts")).collect();
            if let Some(home) = std::env::var_os("HOME") { fuentes.push(PathBuf::from(home).join(".fonts")); }
            fuentes
        }
    }
}

fn propia(ent: &Entorno, path: &Path) -> bool {
    if path.parent() != Some(ent.fuentes_usuario.as_path()) { return false; }
    let Some(name) = path.file_name() else { return false };
    match (fs::symlink_metadata(path), fs::symlink_metadata(ent.activas().join(name))) {
        (Ok(a), Ok(b)) => a.is_file() && b.is_file() && a.dev() == b.dev() && a.ino() == b.ino(),
        _ => false,
    }
}

pub fn entradas_con_directorios(ent: &Entorno, vigiladas: &[PathBuf], dirs: &mut Vec<PathBuf>) -> Vec<Entrada> {
    let mut entradas = Vec::new();
    let mut vistas = HashSet::new();
    let mut agregar = |ruta: PathBuf, origen, activa| {
        if vistas.insert(ruta.clone()) { entradas.push(Entrada { ruta, origen, activa, valor: None }); }
    };
    for ruta in listar(&ent.fuentes_usuario, true) {
        let origen = if propia(ent, &ruta) { Origen::Biblioteca } else { Origen::Usuario };
        agregar(ruta, origen, true);
    }
    for dir in ent.directorios_sistema() {
        for ruta in listar(&dir, true) { agregar(ruta, Origen::Sistema, true); }
    }
    for dir in vigiladas {
        for ruta in listar_con_directorios(dir, true, dirs) { agregar(ruta, Origen::Carpeta, false); }
    }
    for ruta in listar(&ent.biblioteca(), true) { agregar(ruta, Origen::Biblioteca, false); }
    entradas
}

pub mod activacion {
    use super::*;

    pub(super) fn activar(ent: &Entorno, path: &Path) -> io::Result<()> {
        fs::create_dir_all(&ent.fuentes_usuario)?;
        let owned = ent.activas();
        fs::create_dir_all(&owned)?;
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("ttf");
        let mut tmp = tempfile::Builder::new().prefix("FontTime-").suffix(&format!(".{ext}")).tempfile_in(&owned)?;
        io::copy(&mut fs::File::open(path)?, &mut tmp)?;
        tmp.as_file().sync_all()?;
        let (_, anchor) = tmp.keep()?;
        let result = fs::hard_link(&anchor, ent.fuentes_usuario.join(anchor.file_name().unwrap()));
        if result.is_err() { let _ = fs::remove_file(&anchor); }
        result
    }

    pub(super) fn desactivar(ent: &Entorno, path: &Path) -> io::Result<()> {
        if !propia(ent, path) { return Err(io::Error::new(io::ErrorKind::PermissionDenied, "External font")); }
        fs::remove_file(path)?;
        fs::remove_file(ent.activas().join(path.file_name().unwrap()))
    }

    pub(super) fn notificar(ent: &Entorno) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            let status = std::process::Command::new("fc-cache").arg("-f").arg(&ent.fuentes_usuario)
                .status().map_err(|e| e.to_string())?;
            if !status.success() { return Err("Fontconfig could not refresh the user font cache".into()); }
        }
        #[cfg(target_os = "macos")]
        let _ = ent; // fontd monitors the user's Library/Fonts directory.
        Ok(())
    }

    pub fn desactivar_propias(ent: &Entorno) -> Result<(), String> {
        let mut errores = Vec::new();
        let mut cambio = false;
        for p in listar(&ent.fuentes_usuario, false) {
            if propia(ent, &p) {
                match desactivar(ent, &p) { Ok(()) => cambio = true, Err(e) => errores.push(e.to_string()) }
            }
        }
        if cambio { if let Err(e) = notificar(ent) { errores.push(e); } }
        if errores.is_empty() { Ok(()) } else { Err(errores.join("\n")) }
    }
}

pub mod cambios {
    use super::*;
    pub use super::super::resultado::{libre, Resultado};

    pub fn alternar(ent: &Entorno, entradas: &[Entrada], cache: &Cache, familias: &[String], activar: bool, _: bool) -> Resultado {
        let elegidas: HashSet<_> = familias.iter().map(String::as_str).collect();
        let mut archivos = HashSet::new();
        let mut resultado = Resultado::default();
        for familia in cache.familias.iter().filter(|f| elegidas.contains(f.nombre.as_str())) {
            if familia.externa { resultado.bloqueadas += 1; continue; }
            if activar {
                let instalados: HashSet<_> = entradas.iter().filter(|e| e.activa).filter_map(|e| cache.caras(&e.ruta)).flatten()
                    .map(|c| c.completo).collect();
                for estilo in &familia.estilos {
                    if !instalados.contains(&estilo.completo) { archivos.insert(PathBuf::from(&estilo.archivo)); }
                }
            } else {
                archivos.extend(familia.archivos.iter().map(PathBuf::from).filter(|p| propia(ent, p)));
            }
        }
        for path in archivos {
            let r = if activar { activacion::activar(ent, &path) } else { activacion::desactivar(ent, &path) };
            match r {
                Ok(()) => resultado.cambiadas += 1,
                Err(e) => resultado.errores.push(format!("{}: {e}", path.display())),
            }
        }
        if resultado.cambiadas > 0 {
            if let Err(e) = activacion::notificar(ent) { resultado.errores.push(e); }
        }
        resultado
    }
}
