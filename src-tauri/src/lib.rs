mod bandeja;
mod inicio;
pub mod fuentes;
mod google;
pub mod win;

use fuentes::{cache::Cache, cambios, Entorno, Familia, Origen};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::{mpsc, Mutex},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State, WebviewUrl, WebviewWindowBuilder};

/// Mismo archivo que usa tauri-plugin-store desde la interfaz (ver src/lib/ajustes.svelte.js).
const AJUSTES: &str = "ajustes.json";

/// Fondo de la ventana de cada tema (--win en src/app.css): así no hay
/// destello blanco mientras carga la página.
const FONDO_CLARO: (u8, u8, u8, u8) = (0xf7, 0xf6, 0xf3, 255);
const FONDO_OSCURO: (u8, u8, u8, u8) = (0x19, 0x18, 0x16, 255);

fn err(e: impl ToString) -> String {
    e.to_string()
}

pub struct Estado {
    ent: Entorno,
    cache: Mutex<Cache>,
    /// Carpetas vigiladas.
    carpetas: Mutex<Vec<PathBuf>>,
    operaciones: Mutex<()>,
    vigilante: Mutex<Option<notify::RecommendedWatcher>>,
}

impl Estado {
    fn archivo_cache(&self) -> PathBuf {
        self.ent.datos.join("fonts-cache.json")
    }
    fn archivo_carpetas(&self) -> PathBuf {
        self.ent.datos.join("carpetas.json")
    }
    fn entradas(&self, directorios: &mut Vec<PathBuf>) -> Vec<fuentes::Entrada> {
        let carpetas = self.carpetas.lock().unwrap().clone();
        fuentes::entradas_con_directorios(&self.ent, &carpetas, directorios)
    }

    fn actualizar_cache(&self, entradas: &[fuentes::Entrada], cache: &mut Cache) -> usize {
        let leidas = cache.actualizar(entradas);
        let mut catalogadas = HashSet::new();
        let mut propias = HashSet::new();
        let mut externas = HashSet::new();
        let dir = self.ent.activas();
        for e in entradas {
            let grupo = if e.ruta.parent() == Some(dir.as_path()) { &mut propias }
                else if e.origen == Origen::Carpeta { &mut catalogadas }
                else if e.activa { &mut externas } else { continue };
            if let Some(caras) = cache.caras(&e.ruta) {
                grupo.extend(caras.into_iter().map(|c| c.familia.to_lowercase()));
            }
        }
        let visibles = if catalogadas.is_empty() { HashSet::new() } else { win::familias_visibles() };
        for f in &mut cache.familias {
            let nombre = f.nombre.to_lowercase();
            if catalogadas.contains(&nombre) && (externas.contains(&nombre)
                || (!propias.contains(&nombre) && f.estilos.iter().any(|e| visibles.contains(&e.familia_win.to_lowercase())))) {
                f.activa = true;
                f.externa = true;
            }
        }
        leidas
    }
    fn refrescar(&self) -> (Vec<Familia>, usize, Vec<PathBuf>) {
        let mut cache = self.cache.lock().unwrap();
        let mut directorios = Vec::new();
        let entradas = self.entradas(&mut directorios);
        let leidas = self.actualizar_cache(&entradas, &mut cache);
        cache.guardar(&self.archivo_cache());
        (cache.familias.clone(), leidas, directorios)
    }

}

async fn fondo<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(err)?
}

#[derive(Serialize)]
struct Salida {
    familias: Vec<Familia>,
    resultado: cambios::Resultado,
}

/// La lista guardada de la última vez: se pinta al instante.
#[tauri::command]
fn fuentes_guardadas(estado: State<Estado>) -> Vec<Familia> {
    estado.cache.lock().unwrap().familias.clone()
}

/// Comprueba en segundo plano qué ha cambiado y lee solo eso.
#[tauri::command]
async fn fuentes_actualizar(app: AppHandle) -> Result<Value, String> {
    fondo(move || {
        let (familias, leidas, directorios) = app.state::<Estado>().refrescar();
        Ok(json!({ "familias": familias, "leidas": leidas, "directorios": directorios }))
    })
    .await
}

#[tauri::command]
async fn alternar(app: AppHandle, familias: Vec<String>, activar: bool, desbloqueadas: bool) -> Result<Salida, String> {
    fondo(move || {
        let estado = app.state::<Estado>();
        let _op = estado.operaciones.lock().unwrap();
        let entradas = estado.entradas(&mut Vec::new());
        let resultado = {
            let mut cache = estado.cache.lock().unwrap();
            estado.actualizar_cache(&entradas, &mut cache);
            cambios::alternar(&estado.ent, &entradas, &cache, &familias, activar, desbloqueadas)
        };
        let (familias, _, _) = estado.refrescar();
        Ok(Salida { familias, resultado })
    })
    .await
}

/// Importa archivos sueltos al catálogo local; las carpetas vigiladas se leen en su sitio.
#[tauri::command]
async fn agregar_archivos(app: AppHandle, rutas: Vec<PathBuf>) -> Result<Salida, String> {
    fondo(move || {
        let estado = app.state::<Estado>();
        let _op = estado.operaciones.lock().unwrap();
        let resultado = catalogar(&estado.ent, &rutas);
        let (familias, _, _) = estado.refrescar();
        Ok(Salida { familias, resultado })
    }).await
}

fn catalogar(ent: &Entorno, rutas: &[PathBuf]) -> cambios::Resultado {
    let mut resultado = cambios::Resultado::default();
    for ruta in rutas {
        let agregar = || -> Result<String, String> {
            if !fuentes::es_fuente(ruta) { return Err("Unsupported font format".into()); }
            let caras = fuentes::leer::caras(ruta)?;
            let familia = caras.first().ok_or("No font faces")?.familia.clone();
            let nombre = ruta.file_name().ok_or("Missing filename")?.to_string_lossy();
            let dir = ent.biblioteca();
            fs::create_dir_all(&dir).map_err(err)?;
            let base = dir.join(nombre.as_ref());
            if base.is_file() && fs::read(&base).map_err(err)? == fs::read(ruta).map_err(err)? {
                return Ok(familia);
            }
            let destino = cambios::libre(&dir, &nombre);
            if let Err(e) = fs::copy(ruta, &destino) {
                let _ = fs::remove_file(destino);
                return Err(err(e));
            }
            Ok(familia)
        };
        match agregar() {
            Ok(f) => {
                resultado.cambiadas += 1;
                if !resultado.instaladas.contains(&f) { resultado.instaladas.push(f); }
            }
            Err(e) => resultado.errores.push(format!("{}: {e}", ruta.display())),
        }
    }
    resultado
}

/// Ajustes → «restaurar todas las del sistema».
#[tauri::command]
async fn restaurar_sistema(app: AppHandle) -> Result<Salida, String> {
    fondo(move || {
        let estado = app.state::<Estado>();
        let ops: Vec<_> = fuentes::respaldadas(&estado.ent)
            .into_iter()
            .map(|g| fuentes::sistema::Op::Activar { respaldo: g.archivo, ruta: g.original, valor: g.valor, datos: g.datos })
            .collect();
        let mut r = cambios::Resultado::default();
        if !ops.is_empty() {
            cambios::aplicar_sistema(&estado.ent, &ops, &mut r);
            win::avisar_cambio();
        }
        let (familias, _, _) = estado.refrescar();
        Ok(Salida { familias, resultado: r })
    })
    .await
}

#[tauri::command]
fn respaldadas(estado: State<Estado>) -> usize {
    fuentes::respaldadas(&estado.ent).len()
}

/// Caracteres reales de la fuente (tabla cmap), para el mapa del espécimen.
#[tauri::command]
async fn caracteres(ruta: String) -> Result<Vec<u32>, String> {
    fondo(move || fuentes::leer::caracteres(Path::new(&ruta), 4000)).await
}

// ---- carpetas vigiladas ----

fn guardar_carpetas(estado: &Estado) {
    let c: Vec<String> = estado.carpetas.lock().unwrap().iter().map(|p| p.to_string_lossy().into_owned()).collect();
    let _ = fs::create_dir_all(&estado.ent.datos);
    let _ = fs::write(estado.archivo_carpetas(), serde_json::to_vec(&c).unwrap_or_default());
}

fn vigilar(app: &AppHandle) {
    use notify::Watcher;
    let estado = app.state::<Estado>();
    let (tx, rx) = mpsc::channel::<()>();
    let w = notify::recommended_watcher(move |ev: notify::Result<notify::Event>| {
        if let Ok(ev) = ev {
            if matches!(ev.kind, notify::EventKind::Create(_) | notify::EventKind::Remove(_) | notify::EventKind::Modify(notify::event::ModifyKind::Name(_)))
                || ev.paths.iter().any(|p| fuentes::es_fuente(p) || p.extension().is_none()) {
                let _ = tx.send(());
            }
        }
    });
    let Ok(mut w) = w else { return };
    for c in estado.carpetas.lock().unwrap().iter() {
        let _ = w.watch(c, notify::RecursiveMode::Recursive);
    }
    *estado.vigilante.lock().unwrap() = Some(w);
    let app = app.clone();
    std::thread::spawn(move || {
        // Agrupa los cambios seguidos (copiar diez archivos = un aviso).
        while rx.recv().is_ok() {
            while rx.recv_timeout(Duration::from_millis(500)).is_ok() {}
            let _ = app.emit("fuentes-cambiadas", ());
        }
    });
}

#[derive(Serialize)]
struct Carpeta {
    ruta: String,
    nombre: String,
}

fn lista_carpetas(estado: &Estado) -> Vec<Carpeta> {
    estado
        .carpetas
        .lock()
        .unwrap()
        .iter()
        .map(|p| {
            let comps: Vec<String> = p.components().rev().take(2).map(|c| c.as_os_str().to_string_lossy().to_lowercase()).collect();
            let nombre = comps.into_iter().rev().collect::<Vec<_>>().join("\\");
            Carpeta { ruta: p.to_string_lossy().into_owned(), nombre }
        })
        .collect()
}

#[tauri::command]
fn carpetas(estado: State<Estado>) -> Vec<Carpeta> {
    lista_carpetas(&estado)
}

#[tauri::command]
async fn anadir_carpeta(app: AppHandle) -> Result<Option<Value>, String> {
    use tauri_plugin_dialog::DialogExt;
    let Some(elegida) = app.dialog().file().blocking_pick_folder() else { return Ok(None) };
    let ruta = elegida.into_path().map_err(err)?;
    let estado = app.state::<Estado>();
    {
        let mut c = estado.carpetas.lock().unwrap();
        if !c.iter().any(|x| fuentes::clave_ruta(x) == fuentes::clave_ruta(&ruta)) {
            c.push(ruta.clone());
        }
    }
    let _ = app.asset_protocol_scope().allow_directory(&ruta, true);
    guardar_carpetas(&estado);
    vigilar(&app);
    Ok(Some(json!({ "ruta": ruta, "carpetas": lista_carpetas(&estado) })))
}

/// En segundo plano: si aún se están leyendo las fuentes de una carpeta grande,
/// refrescar espera a que acabe, y en el hilo principal eso colgaba la ventana.
#[tauri::command]
async fn quitar_carpeta(app: AppHandle, ruta: String) -> Result<Vec<Carpeta>, String> {
    fondo(move || {
        let estado = app.state::<Estado>();
        estado.carpetas.lock().unwrap().retain(|c| fuentes::clave_ruta(c) != fuentes::clave_ruta(Path::new(&ruta)));
        guardar_carpetas(&estado);
        vigilar(&app);
        // Descarga las de esa carpeta (refrescar ya no las ve).
        let _ = estado.refrescar();
        Ok(lista_carpetas(&estado))
    })
    .await
}

// ---- google fonts ----

#[tauri::command]
async fn google_catalogo(app: AppHandle, forzar: bool) -> Result<Value, String> {
    let datos = app.state::<Estado>().ent.datos.clone();
    let lista = google::catalogo(&datos, forzar).await?;
    serde_json::to_value(lista).map_err(err)
}

#[tauri::command]
async fn google_instalar(app: AppHandle, familia: String) -> Result<Salida, String> {
    let datos = app.state::<Estado>().ent.datos.clone();
    let destino = datos.join("descargas").join(familia.replace(|c: char| !c.is_alphanumeric(), "_"));
    let a = app.clone();
    let f = familia.clone();
    let mut ultimo = -1.0;
    let archivos = google::bajar(&familia, &destino, move |p| {
        if p - ultimo >= 2.0 {
            ultimo = p;
            let _ = a.emit("descarga", json!({ "familia": f, "pct": p }));
        }
    })
    .await;
    let archivos = match archivos {
        Ok(a) => a,
        Err(e) => {
            let _ = app.emit("descarga", json!({ "familia": familia, "error": true }));
            return Err(e);
        }
    };
    let app2 = app.clone();
    let salida = fondo(move || {
        let estado = app2.state::<Estado>();
        let _op = estado.operaciones.lock().unwrap();
        let r = catalogar(&estado.ent, &archivos);
        let _ = fs::remove_dir_all(&destino);
        let (familias, _, _) = estado.refrescar();
        Ok(Salida { familias, resultado: r })
    })
    .await?;
    let _ = app.emit("descarga", json!({ "familia": familia, "pct": 100 }));
    Ok(salida)
}

// ---- actualizaciones y enlaces ----

/// La portable no se puede actualizar sola (el actualizador instalaría la app):
/// en ella el aviso lleva a la página de la release. La instalada tiene su desinstalador al lado.
#[tauri::command]
fn es_portable() -> bool {
    std::env::current_exe().ok().and_then(|e| e.parent().map(|d| !d.join("uninstall.exe").is_file())).unwrap_or(true)
}

/// Abre en el navegador una de estas direcciones fijas: la interfaz no puede pedir otra.
#[tauri::command]
fn abrir_web(cual: String) -> Result<(), String> {
    let url = match cual.as_str() {
        "release" => "https://github.com/eugilvando/font-time/releases/latest",
        "fallo" => "https://github.com/eugilvando/font-time/issues/new/choose",
        "repo" => "https://github.com/eugilvando/font-time",
        _ => return Err(format!("dirección desconocida: {cual}")),
    };
    // Con ShellExecute y no con «explorer url»: el Explorador no reconoce como web
    // una dirección con parámetros (?a=1&b=2) y abría la carpeta Documentos.
    win::abrir_url(url)
}

/// Abre el Explorador con el archivo de una fuente seleccionado. Solo acepta
/// archivos de fuente que existan: la interfaz no puede abrir otra cosa.
#[tauri::command]
fn mostrar_en_explorador(ruta: String) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let p = PathBuf::from(&ruta);
    let es_fuente = p
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc" | "otc"));
    if !es_fuente || !p.is_file() || ruta.contains('"') {
        return Err(format!("no es un archivo de fuente: {ruta}"));
    }
    // raw_arg: explorer interpreta él mismo «/select,"ruta"», con comillas por los espacios.
    std::process::Command::new("explorer")
        .raw_arg(format!("/select,\"{ruta}\""))
        .spawn()
        .map(|_| ())
        .map_err(err)
}

// ---- arranque ----

fn leer_ajustes(app: &AppHandle) -> Value {
    use tauri_plugin_store::StoreExt;
    if let Some(store) = app.get_store(AJUSTES) {
        return Value::Object(store.entries().into_iter().collect());
    }
    app.path()
        .app_data_dir()
        .ok()
        .and_then(|d| fs::read_to_string(d.join(AJUSTES)).ok())
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}))
}

/// Antes de la primera línea de la página: los ajustes guardados y el tema
/// puesto en <html>, para que el primer fotograma ya salga con su color e idioma.
fn script_inicio(inicio: &Value) -> String {
    format!(
        "window.__TUNA__ = {inicio};
        (function (t) {{
          if (t !== 'light' && t !== 'dark')
            t = matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
          var d = document, s = function () {{ d.documentElement.dataset.theme = t; }};
          if (d.documentElement) s();
          else new MutationObserver(function (m, o) {{
            if (d.documentElement) {{ s(); o.disconnect(); }}
          }}).observe(d, {{ childList: true }});
        }})(window.__TUNA__.ajustes.tema);"
    )
}

/// Tema oscuro de Windows (apps), para la primera vez que se abre.
fn windows_oscuro() -> bool {
    winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|k| k.get_value::<u32, _>("AppsUseLightTheme"))
        .map(|v| v == 0)
        .unwrap_or(false)
}

/// Idioma de Windows para el primer arranque: "es" si la interfaz del sistema
/// está en español (cualquier variante), "en" para todo lo demás.
fn idioma_windows() -> &'static str {
    // SAFETY: sin argumentos; devuelve el LANGID de la interfaz del usuario.
    let langid = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
    match langid & 0x3ff { 0x16 => "pt-BR", 0x0a => "es", _ => "en" }
}

/// Con `--elevado <peticion.json>`: la copia con permisos de administrador.
pub fn modo_elevado() -> Option<i32> {
    let mut args = std::env::args_os().skip(1);
    if args.next()?.to_str()? != "--elevado" {
        return None;
    }
    let peticion = PathBuf::from(args.next()?);
    Some(fuentes::sistema::atender(&peticion))
}

pub fn run() {
    let app = tauri::Builder::default()
        // Una sola tuna-fonts a la vez: dos moviendo las mismas fuentes se pisarían.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| bandeja::mostrar(app)))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        // La X y Alt+F4: a la bandeja (las carpetas vigiladas siguen disponibles),
        // salvo que en ajustes se haya elegido cerrar del todo.
        .on_window_event(|w, ev| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = ev {
                if w.label() == "main" {
                    api.prevent_close();
                    if leer_ajustes(w.app_handle())["alCerrar"].as_str() == Some("salir") { w.app_handle().exit(0); }
                    else { let _ = w.hide(); }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            fuentes_guardadas,
            fuentes_actualizar,
            alternar,
            agregar_archivos,
            restaurar_sistema,
            respaldadas,
            caracteres,
            carpetas,
            anadir_carpeta,
            quitar_carpeta,
            google_catalogo,
            google_instalar,
            es_portable,
            abrir_web,
            bandeja::poner_bandeja,
            bandeja::estado_bandeja,
            bandeja::accion_bandeja,
            inicio::inicio_windows,
            inicio::poner_inicio_windows,
            inicio::marcar_mostrar_al_abrir,
            mostrar_en_explorador,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let datos = app.path().app_data_dir()?;
            // Con Windows (--oculta) se queda en la bandeja; tras actualizarse, no.
            let oculta = inicio::arrancar_oculta(&datos);
            let ent = Entorno::real(datos.clone());
            let cache = Cache::cargar(&datos.join("fonts-cache.json"));
            let carpetas: Vec<PathBuf> = fs::read(datos.join("carpetas.json"))
                .ok()
                .and_then(|b| serde_json::from_slice::<Vec<String>>(&b).ok())
                .unwrap_or_default()
                .into_iter()
                .map(PathBuf::from)
                .collect();

            // Vistas previas de las que no están instaladas ahora (desactivadas,
            // de carpetas, recién instaladas): se cargan desde su archivo.
            let alcance = app.asset_protocol_scope();
            for d in [ent.desactivadas(), ent.respaldo(), ent.activas(), ent.biblioteca(), ent.fuentes_usuario.clone(), ent.fuentes_sistema.clone()] {
                let _ = alcance.allow_directory(d, true);
            }
            for c in &carpetas {
                let _ = alcance.allow_directory(c, true);
            }

            app.manage(Estado {
                ent,
                cache: Mutex::new(cache),
                carpetas: Mutex::new(carpetas),
                operaciones: Mutex::default(),
                vigilante: Mutex::default(),
            });
            vigilar(&handle);

            let ajustes = leer_ajustes(&handle);
            let oscuro = match ajustes["tema"].as_str() {
                Some("dark") => true,
                Some("light") => false,
                _ => windows_oscuro(),
            };
            let tema = if oscuro { "dark" } else { "light" };
            bandeja::crear(&handle, tema, ajustes["idioma"].as_str().unwrap_or(idioma_windows()))?;
            let inicio = json!({
                "ajustes": ajustes,
                "version": app.package_info().version.to_string(),
                "idiomaSistema": idioma_windows(),
                "actualizador": app.config().plugins.0.get("updater")
                    .and_then(|cfg| cfg["pubkey"].as_str()).is_some_and(|key| !key.is_empty()),
            });

            let principal = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("Font Time")
                .visible(!oculta)
                .inner_size(1280.0, 820.0)
                .min_inner_size(960.0, 600.0)
                .center()
                .decorations(false)
                .shadow(true)
                .zoom_hotkeys_enabled(false)
                .background_color(if oscuro { FONDO_OSCURO } else { FONDO_CLARO }.into())
                .initialization_script(script_inicio(&inicio))
                .build()?;
            win::poner_iconos_ventana(principal.hwnd()?.0)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error al arrancar Font Time");

    app.run(|app, ev| {
        if let RunEvent::Exit = ev {
            let estado = app.state::<Estado>();
            let _op = estado.operaciones.lock().unwrap();
            if leer_ajustes(app)["desactivarAlSalir"].as_bool() == Some(true) {
                if let Err(e) = fuentes::activacion::desactivar_propias(&estado.ent) { eprintln!("{e}"); }
            }
        }
    });
}
