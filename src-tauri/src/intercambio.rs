use crate::{err, fondo, Estado};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

const MAX: u64 = 16 * 1024 * 1024;

fn leer(path: &Path) -> Result<String, String> {
    let mut buf = Vec::new();
    fs::File::open(path)
        .map_err(err)?
        .take(MAX + 1)
        .read_to_end(&mut buf)
        .map_err(err)?;
    if buf.len() as u64 > MAX {
        return Err("File exceeds 16 MiB".into());
    }
    String::from_utf8(buf).map_err(err)
}

fn temporal(path: &Path) -> Result<tempfile::NamedTempFile, String> {
    tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing directory")?).map_err(err)
}

pub(crate) fn escribir(path: &Path, contenido: &[u8]) -> Result<(), String> {
    let mut temp = temporal(path)?;
    temp.write_all(contenido).map_err(err)?;
    temp.as_file().sync_all().map_err(err)?;
    temp.persist(path).map_err(err)?;
    Ok(())
}

#[tauri::command]
pub async fn importar_colecciones(app: AppHandle) -> Result<Option<String>, String> {
    let Some(p) = app
        .dialog()
        .file()
        .add_filter("Font Time", &["json"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    leer(&p.into_path().map_err(err)?).map(Some)
}

#[tauri::command]
pub async fn exportar_colecciones(
    app: AppHandle,
    contenido: String,
    archivo: bool,
    familias: Vec<String>,
) -> Result<bool, String> {
    let ext = if archivo { "zip" } else { "json" };
    let Some(file) = app
        .dialog()
        .file()
        .add_filter("Font Time", &[ext])
        .set_file_name(format!("font-time-collections.{ext}"))
        .blocking_save_file()
    else {
        return Ok(false);
    };
    let path = file.into_path().map_err(err)?;
    fondo(move || {
        if !archivo {
            escribir(&path, contenido.as_bytes())?;
            return Ok(true);
        }
        let mut temp = temporal(&path)?;
        let elegidas: BTreeSet<_> = familias.into_iter().collect();
        let rutas: BTreeSet<String> = app
            .state::<Estado>()
            .cache
            .lock()
            .unwrap()
            .familias
            .iter()
            .filter(|f| elegidas.contains(&f.nombre) && f.error.is_none())
            .flat_map(|f| f.archivos.iter().cloned())
            .collect();
        {
            let mut zip = zip::ZipWriter::new(temp.as_file_mut());
            let op = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            zip.start_file("collections.json", op).map_err(err)?;
            zip.write_all(contenido.as_bytes()).map_err(err)?;
            for (i, ruta) in rutas.iter().enumerate() {
                let p = Path::new(ruta);
                let nombre = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or("Invalid font filename")?;
                zip.start_file(format!("fonts/{i}/{nombre}"), op)
                    .map_err(err)?;
                let mut input = fs::File::open(p).map_err(err)?;
                std::io::copy(&mut input, &mut zip).map_err(err)?;
            }
            zip.finish().map_err(err)?;
        }
        temp.as_file().sync_all().map_err(err)?;
        temp.persist(path).map_err(err)?;
        Ok(true)
    })
    .await
}

fn ruta_config(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(err)?
        .join("collections-sync-path.json"))
}
fn carpeta_sync(app: &AppHandle) -> Result<Option<PathBuf>, String> {
    let path = ruta_config(app)?;
    match fs::read(path) {
        Ok(b) => serde_json::from_slice(&b).map_err(err),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(err(e)),
    }
}

#[tauri::command]
pub async fn configurar_sync(app: AppHandle, accion: String) -> Result<Option<PathBuf>, String> {
    let dir = match accion.as_str() {
        "pick" => {
            let Some(p) = app.dialog().file().blocking_pick_folder() else {
                return carpeta_sync(&app);
            };
            let dir = p.into_path().map_err(err)?.join("Font Time Collections");
            fs::create_dir_all(&dir).map_err(err)?;
            Some(dir)
        }
        "off" => None,
        "get" => return carpeta_sync(&app),
        _ => return Err("Unknown action".into()),
    };
    let config = ruta_config(&app)?;
    fs::create_dir_all(config.parent().unwrap()).map_err(err)?;
    escribir(&config, &serde_json::to_vec(&dir).map_err(err)?)?;
    Ok(dir)
}

#[tauri::command]
pub async fn diario_sync(
    app: AppHandle,
    carpeta: PathBuf,
    guardar: Option<Value>,
) -> Result<Vec<Value>, String> {
    fondo(move || {
        if carpeta_sync(&app)?.as_ref() != Some(&carpeta) {
            return Err("Sync folder changed".into());
        }
        if let Some(valor) = guardar {
            let id = valor["id"].as_str().ok_or("Missing batch ID")?;
            if id.len() != 36 || !id.bytes().all(|c| c.is_ascii_hexdigit() || c == b'-') {
                return Err("Invalid batch ID".into());
            }
            let b = serde_json::to_vec(&valor).map_err(err)?;
            if b.len() as u64 > MAX {
                return Err("Batch too large".into());
            }
            let path = carpeta.join(format!("ft-{id}.json"));
            let mut tmp = temporal(&path)?;
            tmp.write_all(&b).map_err(err)?;
            tmp.as_file().sync_all().map_err(err)?;
            if let Err(e) = tmp.persist_noclobber(&path) {
                if e.error.kind() != std::io::ErrorKind::AlreadyExists
                    || fs::read(&path).map_err(err)? != b
                {
                    return Err(err(e));
                }
            }
            return Ok(Vec::new());
        }
        let mut result = Vec::new();
        for entry in fs::read_dir(carpeta).map_err(err)? {
            let e = entry.map_err(err)?;
            let name = e.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("ft-")
                || !name.ends_with(".json")
                || !e.file_type().map_err(err)?.is_file()
            {
                continue;
            }
            result.push(serde_json::from_str(&leer(&e.path())?).map_err(err)?);
        }
        Ok(result)
    })
    .await
}

#[tauri::command]
pub async fn etiquetas_publicas(
    app: AppHandle,
) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let cache = app
        .path()
        .app_cache_dir()
        .map_err(err)?
        .join("google-font-tags.csv");
    let anterior = fs::metadata(&cache)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|m| m.elapsed().ok());
    let mut csv = leer(&cache).ok();
    if csv.is_none() || anterior.is_none_or(|a| a.as_secs() > 7 * 86400) {
        let nuevo = async {
            let body = crate::google::cliente()?
                .get("https://raw.githubusercontent.com/google/fonts/main/tags/all/families.csv")
                .timeout(std::time::Duration::from_secs(20))
                .send()
                .await
                .map_err(err)?
                .error_for_status()
                .map_err(err)?
                .text()
                .await
                .map_err(err)?;
            if body.len() as u64 > MAX || !body.contains(",/") {
                return Err("Invalid Google Fonts tag file".to_owned());
            }
            Ok(body)
        }
        .await;
        match nuevo {
            Ok(body) => {
                fs::create_dir_all(cache.parent().unwrap()).map_err(err)?;
                escribir(&cache, body.as_bytes())?;
                csv = Some(body);
            }
            Err(e) if csv.is_none() => return Err(e),
            Err(_) => {}
        }
    }
    let mut tags = BTreeMap::<String, BTreeSet<String>>::new();
    for line in csv.unwrap_or_default().lines() {
        let mut campos = line.rsplitn(3, ',');
        let score = campos
            .next()
            .and_then(|v| v.trim().parse::<f32>().ok())
            .unwrap_or_default();
        let tag = campos
            .next()
            .and_then(|v| v.rsplit('/').next())
            .unwrap_or("");
        let family = campos
            .next()
            .and_then(|v| v.rsplit_once(','))
            .map(|(f, _)| f.trim_matches('"'));
        if score >= 50. && !tag.is_empty() {
            if let Some(f) = family {
                tags.entry(f.to_owned()).or_default().insert(tag.to_owned());
            }
        }
    }
    Ok(tags)
}

#[tauri::command]
pub async fn ejes_fuente(
    app: AppHandle,
    ruta: String,
    familia: String,
) -> Result<Vec<Value>, String> {
    fondo(move || {
        let known = app
            .state::<Estado>()
            .cache
            .lock()
            .unwrap()
            .familias
            .iter()
            .any(|f| f.nombre == familia && f.archivos.contains(&ruta) && f.error.is_none());
        if !known {
            return Ok(Vec::new());
        }
        crate::fuentes::leer::ejes(Path::new(&ruta), &familia)
    })
    .await
}
