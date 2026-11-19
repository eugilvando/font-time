use crate::{
    err, fondo,
    fuentes::{cambios, Origen},
    intercambio::escribir,
    Estado,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, path::Path, time::Duration};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

#[derive(Default, Deserialize, Serialize)]
struct Control {
    propias: BTreeSet<String>,
    ignoradas: BTreeSet<String>,
}

fn control(dir: &Path) -> Result<Control, String> {
    match fs::read(dir.join("adobe-control.json")) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(err),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Control::default()),
        Err(e) => Err(err(e)),
    }
}
fn guardar(dir: &Path, c: &Control) -> Result<(), String> {
    escribir(
        &dir.join("adobe-control.json"),
        &serde_json::to_vec(c).map_err(err)?,
    )
}
pub fn manual(dir: &Path, nombres: &[String], activar: bool) -> Result<(), String> {
    if !dir.join("adobe-control.json").is_file() {
        return Ok(());
    }
    let mut c = control(dir)?;
    for n in nombres {
        c.propias.remove(n);
        if activar {
            c.ignoradas.remove(n);
        } else {
            c.ignoradas.insert(n.clone());
        }
    }
    guardar(dir, &c)
}

#[tauri::command]
pub async fn adobe_mantener(app: AppHandle, familia: String) -> Result<(), String> {
    fondo(move || {
        let estado = app.state::<Estado>();
        let _guard = estado.operaciones.lock().unwrap();
        manual(&estado.ent.datos, &[familia], true)
    })
    .await
}

#[tauri::command]
pub async fn adobe_script(app: AppHandle) -> Result<bool, String> {
    let dir = app.path().app_data_dir().map_err(err)?.join("adobe");
    let Some(p) = app
        .dialog()
        .file()
        .set_file_name("Font Time.jsx")
        .add_filter("Adobe Bridge", &["jsx"])
        .blocking_save_file()
    else {
        return Ok(false);
    };
    let script = include_str!("../../integrations/adobe/Font Time.jsx").replace(
        "__FONT_TIME_DIRECTORY__",
        &serde_json::to_string(&dir).map_err(err)?,
    );
    escribir(&p.into_path().map_err(err)?, script.as_bytes())?;
    Ok(true)
}

#[tauri::command]
pub async fn adobe_actualizar(app: AppHandle, cerrar: bool) -> Result<serde_json::Value, String> {
    fondo(move || {
        let estado=app.state::<Estado>();
        let _op=estado.operaciones.lock().unwrap();
        let base=&estado.ent.datos;
        let dir=base.join("adobe");
        fs::create_dir_all(&dir).map_err(err)?;
        fs::write(dir.join("enabled"),b"1").map_err(err)?;
        let mut activas=BTreeSet::new();
        let mut vivos=0;
        for host in ["indesign","photoshop","illustrator"] {
            let path=dir.join(format!("{host}.txt"));
            let Ok(m)=path.metadata() else{continue};
            if m.len()>4*1024*1024 || !m.modified().ok().and_then(|v|v.elapsed().ok()).is_some_and(|e|e < Duration::from_secs(45)) {continue;}
            let Ok(s)=fs::read_to_string(path) else{continue};
            let mut lines=s.trim_start_matches('\u{feff}').lines();
            if lines.next()!=Some("FT1"){continue;}
            vivos+=1;
            for line in lines.filter(|l|!l.is_empty()) {
                if let Ok(n)=percent_encoding::percent_decode_str(line).decode_utf8() {activas.insert(n.to_lowercase());}
            }
        }
        let mut c=control(base)?;
        let mut pedidos=BTreeSet::new();
        let mut activar=Vec::new();
        let mut desactivar=Vec::new();
        {
            let cache=estado.cache.lock().unwrap();
            for f in &cache.familias {
                if f.error.is_some() || !matches!(f.origen,Origen::Biblioteca|Origen::Carpeta) || f.externa {continue;}
                if f.estilos.iter().any(|e| !e.postscript.is_empty() && activas.contains(&e.postscript.to_lowercase())) {
                    pedidos.insert(f.nombre.clone());
                    if !f.activa && !c.ignoradas.contains(&f.nombre) {activar.push(f.nombre.clone());}
                } else if vivos==3 && cerrar && f.activa && c.propias.contains(&f.nombre) {
                    desactivar.push(f.nombre.clone());
                }
            }
        }
        let old=serde_json::to_vec(&c).map_err(err)?;
        if vivos==3 {
            c.ignoradas.retain(|n|pedidos.contains(n));
            if !cerrar {c.propias.retain(|n|pedidos.contains(n));}
        }
        let mut changed=false;
        let mut errores=Vec::new();
        for (nombres,on) in [(activar,true),(desactivar,false)] {
            if nombres.is_empty(){continue;}
            let entradas=estado.entradas(&mut Vec::new());
            let mut cache=estado.cache.lock().unwrap();
            estado.actualizar_cache(&entradas,&mut cache);
            // Recheck external ownership under the same lock as a manual activation.
            let seguras:Vec<_>=nombres.into_iter().filter(|n|cache.familias.iter().any(|f|
                &f.nombre==n && f.activa!=on && !f.externa && matches!(f.origen,Origen::Biblioteca|Origen::Carpeta) && f.error.is_none())).collect();
            if seguras.is_empty(){continue;}
            let res=cambios::alternar(&estado.ent,&entradas,&cache,&seguras,on,false);
            errores.extend(res.errores);
            drop(cache);
            let (lista,_,_)=estado.refrescar();
            for n in seguras {
                if lista.iter().any(|f|f.nombre==n && f.activa==on && !f.externa) {
                    if on {c.propias.insert(n);} else {c.propias.remove(&n);}
                    changed=true;
                }
            }
        }
        if serde_json::to_vec(&c).map_err(err)?!=old { guardar(base,&c)?; }
        Ok(serde_json::json!({"connected":vivos==3,"changed":changed,"errors":errores,"owned":c.propias}))
    }).await
}
