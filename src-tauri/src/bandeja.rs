//! Icono en la bandeja del sistema: cerrar la ventana la esconde y las fuentes
//! de las carpetas vigiladas siguen disponibles. El menú del clic derecho no es
//! el nativo de Windows (que no se puede pintar a nuestro gusto) sino una
//! ventanita propia, src/Menu.svelte, creada al arrancar y escondida hasta que
//! hace falta. Es el mismo que el de tuna-dlp, con el diseño de tuna-fonts.
//!
//! El menú nunca se queda con el foco: si lo hiciera, el desplegable de iconos
//! ocultos de Windows (la flecha) se cerraría al abrirlo. Así que se cierra «a
//! mano»: clic fuera, Esc, o que el desplegable desaparezca.

use serde::Serialize;
use serde_json::json;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    thread,
    time::Duration,
};
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use windows_sys::Win32::{
    Foundation::POINT,
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON},
        WindowsAndMessaging::{GetCursorPos, GetForegroundWindow, IsWindowVisible},
    },
};

use crate::{err, script_inicio};

const ANCHO: f64 = 264.0;
const ALTO: f64 = 292.0;

/// Lo que enseña el menú. Lo manda la ventana principal.
#[derive(Clone, Serialize, Default)]
pub struct Estado {
    estado: String,
    carpetas: u32,
    tema: String,
    idioma: String,
}

#[derive(Default)]
pub struct Bandeja(Mutex<Estado>);

pub fn crear(app: &AppHandle, tema: &str, idioma: &str) -> tauri::Result<()> {
    app.manage(Bandeja(Mutex::new(Estado { tema: tema.into(), idioma: idioma.into(), ..Default::default() })));

    let inicio = json!({ "ajustes": { "tema": tema }, "menu": true });
    let menu = WebviewWindowBuilder::new(app, "menu", WebviewUrl::default())
        .title("Font Time")
        .inner_size(ANCHO, ALTO)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .visible(false)
        // WS_EX_NOACTIVATE: se puede pulsar, pero no roba el foco.
        .focusable(false)
        .zoom_hotkeys_enabled(false)
        .initialization_script(script_inicio(&inicio))
        .build()?;
    // Con Alt+F4 se destruiría y el menú dejaría de funcionar.
    let escondido = menu.clone();
    menu.on_window_event(move |ev| {
        if let WindowEvent::CloseRequested { api, .. } = ev {
            api.prevent_close();
            let _ = escondido.hide();
        }
    });

    let icono = TrayIconBuilder::with_id("font-time").tooltip("Font Time").on_tray_icon_event(|icono, ev| {
        if let TrayIconEvent::Click { button, button_state: MouseButtonState::Up, position, .. } = ev {
            match button {
                MouseButton::Left => mostrar(icono.app_handle()),
                MouseButton::Right => abrir_menu(icono.app_handle(), position),
                _ => {}
            }
        }
    });
    icono.icon(tauri::include_image!("icons/32x32.png")).build(app)?;
    Ok(())
}

/// El menú sale con su esquina inferior derecha en el cursor (la barra de
/// tareas suele estar abajo), sin salirse de la pantalla.
fn abrir_menu(app: &AppHandle, cursor: PhysicalPosition<f64>) {
    let Some(menu) = app.get_webview_window("menu") else { return };
    let pantalla = app.monitor_from_point(cursor.x, cursor.y).ok().flatten().or_else(|| app.primary_monitor().ok().flatten());
    let Some(pantalla) = pantalla else { return };
    let escala = pantalla.scale_factor();
    let zona = pantalla.work_area();
    let (ancho, alto) = (ANCHO * escala, ALTO * escala);
    let (izq, arr) = (zona.position.x as f64, zona.position.y as f64);
    let (der, aba) = (izq + zona.size.width as f64, arr + zona.size.height as f64);
    let x = (cursor.x - ancho).clamp(izq, der - ancho);
    let y = (cursor.y - alto).clamp(arr, aba - alto);
    let _ = menu.set_position(PhysicalPosition::new(x, y));
    let _ = menu.emit("menu-abierto", ());
    let _ = menu.show();
    vigilar(menu, (x, y, ancho, alto));
}

/// Cada apertura del menú tiene su vigilante; el de una apertura anterior se retira.
static APERTURA: AtomicU64 = AtomicU64::new(0);

fn pulsada(tecla: u16) -> bool {
    // SAFETY: sin punteros; el bit alto dice si la tecla está abajo ahora mismo.
    (unsafe { GetAsyncKeyState(tecla as i32) } as u16) & 0x8000 != 0
}

fn boton_pulsado() -> bool {
    [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON].into_iter().any(pulsada)
}

/// Mientras el menú está abierto se vigila a mano, porque no tiene el foco:
/// se cierra con un clic fuera de su rectángulo, con Esc, o cuando desaparece
/// la ventana que tenía el foco al abrirlo (el desplegable de la flecha se
/// cierra, o el usuario se va a otra ventana).
fn vigilar(menu: WebviewWindow, (x, y, ancho, alto): (f64, f64, f64, f64)) {
    let mia = APERTURA.fetch_add(1, Ordering::SeqCst) + 1;
    // SAFETY: sin argumentos.
    let anterior = unsafe { GetForegroundWindow() } as isize;
    thread::spawn(move || {
        let sigue = || APERTURA.load(Ordering::SeqCst) == mia && menu.is_visible().unwrap_or(false);
        let propia = menu.hwnd().map_or(0, |h| h.0 as isize);
        let se_fue = || {
            // SAFETY: GetForegroundWindow no tiene argumentos; IsWindowVisible
            // acepta cualquier valor (devuelve 0 si la ventana ya no existe).
            let ahora = unsafe { GetForegroundWindow() } as isize;
            let visible = anterior == 0 || unsafe { IsWindowVisible(anterior as _) } != 0;
            !visible || (ahora != anterior && ahora != propia && ahora != 0)
        };
        // Primero, que se suelte el clic derecho con el que se abrió.
        while boton_pulsado() {
            if !sigue() {
                return;
            }
            thread::sleep(Duration::from_millis(15));
        }
        while sigue() {
            thread::sleep(Duration::from_millis(20));
            if pulsada(VK_ESCAPE) || se_fue() {
                let _ = menu.hide();
                return;
            }
            if !boton_pulsado() {
                continue;
            }
            let mut p = POINT { x: 0, y: 0 };
            // SAFETY: p es un POINT válido en la pila.
            unsafe { GetCursorPos(&mut p) };
            let (px, py) = (p.x as f64, p.y as f64);
            if px < x || px >= x + ancho || py < y || py >= y + alto {
                let _ = menu.hide();
                return;
            }
            // Clic dentro: el menú se encarga. Se espera a que se suelte.
            while boton_pulsado() && sigue() {
                thread::sleep(Duration::from_millis(15));
            }
        }
    });
}

/// Vuelve a enseñar la ventana (desde la bandeja o al abrir la app otra vez).
pub fn mostrar(app: &AppHandle) {
    if let Some(v) = app.get_webview_window("main") {
        let _ = v.show();
        let _ = v.unminimize();
        let _ = v.set_focus();
    }
}

/// La ventana principal cuenta cómo están las fuentes; el menú lo pinta.
#[tauri::command]
pub fn poner_bandeja(app: AppHandle, estado: String, carpetas: u32, tema: String, idioma: String) {
    let nuevo = Estado { estado, carpetas, tema, idioma };
    if let Some(b) = app.try_state::<Bandeja>() {
        *b.0.lock().unwrap() = nuevo.clone();
    }
    let _ = app.emit_to("menu", "menu-estado", nuevo);
}

#[tauri::command]
pub fn estado_bandeja(app: AppHandle) -> Estado {
    app.try_state::<Bandeja>().map(|b| b.0.lock().unwrap().clone()).unwrap_or_default()
}

/// Lo que se pulsa en el menú. Buscar, descubrir y ajustes los hace la ventana
/// principal, después de enseñarse.
#[tauri::command]
pub fn accion_bandeja(app: AppHandle, accion: String) -> Result<(), String> {
    if let Some(m) = app.get_webview_window("menu") {
        let _ = m.hide();
    }
    match accion.as_str() {
        "abrir" => mostrar(&app),
        // Salir de verdad: RunEvent::Exit quita las fuentes de las carpetas vigiladas.
        "salir" => app.exit(0),
        "cerrar" => {}
        "buscar" | "descubrir" | "ajustes" => {
            mostrar(&app);
            app.emit_to("main", "bandeja", accion).map_err(err)?;
        }
        otra => return Err(format!("acción desconocida: {otra}")),
    }
    Ok(())
}
