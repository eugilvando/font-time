use crate::{err, fondo, fuentes::{self, cache::Cache, Entrada, Origen}, Estado, SeleccionEstilo};
use serde::Serialize;
use std::{collections::BTreeSet, path::{Path,PathBuf}};
use tauri::{AppHandle, Manager};

fn elegibles(estado:&Estado, entradas:&[Entrada], cache:&Cache, familias:&[String], estilos:&[SeleccionEstilo]) -> BTreeSet<PathBuf> {
    let mut dirs=estado.carpetas.lock().unwrap().clone();
    dirs.push(estado.ent.biblioteca());
    let dirs:Vec<_>=dirs.iter().filter_map(|d|d.canonicalize().ok()).collect();
    let mut paths=BTreeSet::new();
    for e in entradas {
        if e.activa || !matches!(e.origen,Origen::Carpeta|Origen::Biblioteca) || !fuentes::es_fuente(&e.ruta) {continue;}
        let Some(caras)=cache.caras(&e.ruta) else { continue };
        if caras.is_empty() || !caras.iter().all(|c| {
            (familias.iter().any(|f| f.eq_ignore_ascii_case(&c.familia)) ||
                estilos.iter().any(|s| s.familia.eq_ignore_ascii_case(&c.familia) && s.nombre==c.estilo && s.peso==c.peso && s.cursiva==c.cursiva)) &&
            cache.familias.iter().find(|f| f.nombre.eq_ignore_ascii_case(&c.familia)).is_some_and(|f|
                !f.externa && f.estilos.iter().any(|s| s.nombre==c.estilo && s.peso==c.peso && s.cursiva==c.cursiva && !s.activa))
        }) { continue; }
        if !e.ruta.symlink_metadata().is_ok_and(|m| m.file_type().is_file()) { continue; }
        if let Ok(p)=e.ruta.canonicalize() {
            if dirs.iter().any(|d| p.starts_with(d)) {paths.insert(e.ruta.clone());}
        }
    }
    paths
}

#[derive(Serialize,Default)]
pub struct Resultado { rutas:Vec<PathBuf>, errores:Vec<String> }

#[tauri::command]
pub async fn papelera(app:AppHandle, familias:Vec<String>, estilos:Vec<SeleccionEstilo>, confirmadas:Option<Vec<PathBuf>>) -> Result<Resultado,String> {
    fondo(move || {
        let estado=app.state::<Estado>();
        let _op=estado.operaciones.lock().unwrap();
        let entradas=estado.entradas(&mut Vec::new());
        let mut cache=estado.cache.lock().unwrap();
        estado.actualizar_cache(&entradas,&mut cache);
        let paths=elegibles(&estado,&entradas,&cache,&familias,&estilos);
        drop(cache);
        let mut r=Resultado::default();
        if let Some(aceptadas)=confirmadas {
            for p in aceptadas.into_iter().collect::<BTreeSet<_>>() {
                if !paths.contains(&p) { r.errores.push(format!("{}: unavailable",p.display()));continue; }
                match trash::delete(Path::new(&p)).map_err(err) {
                    Ok(()) => r.rutas.push(p),
                    Err(e) => r.errores.push(format!("{}: {e}",p.display())),
                }
            }
        } else { r.rutas=paths.into_iter().collect(); }
        Ok(r)
    }).await
}
