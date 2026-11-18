//! Llamadas a Windows: cargar y descargar fuentes en la sesión, avisar a los
//! programas de que han cambiado, mover archivos al reiniciar y relanzarse con
//! permisos de administrador (UAC).

use std::{ffi::OsStr, os::windows::ffi::OsStrExt, path::Path};
use windows_sys::Win32::{
    Foundation::{CloseHandle, WAIT_OBJECT_0},
    Graphics::Gdi::{AddFontResourceExW, RemoveFontResourceExW, FR_PRIVATE},
    Storage::FileSystem::{MoveFileExW, MOVEFILE_DELAY_UNTIL_REBOOT},
    System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE},
    UI::{
        Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW},
        WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, SW_HIDE, WM_FONTCHANGE},
    },
};

/// Ruta de una carpeta conocida (FOLDERID_*), pedida a Windows en vez de
/// montarla a mano con variables de entorno. None si Windows no la da.
pub fn carpeta_conocida(id: &windows_sys::core::GUID) -> Option<std::path::PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::{System::Com::CoTaskMemFree, UI::Shell::SHGetKnownFolderPath};
    let mut ptr: windows_sys::core::PWSTR = std::ptr::null_mut();
    // SAFETY: SHGetKnownFolderPath reserva la cadena; se copia y se libera
    // siempre con CoTaskMemFree, también si falla (puede venir no nula).
    unsafe {
        let hr = SHGetKnownFolderPath(id, 0, std::ptr::null_mut(), &mut ptr);
        let ruta = if hr == 0 && !ptr.is_null() {
            let mut n = 0;
            while *ptr.add(n) != 0 {
                n += 1;
            }
            Some(std::ffi::OsString::from_wide(std::slice::from_raw_parts(ptr, n)).into())
        } else {
            None
        };
        CoTaskMemFree(ptr as *const _);
        ruta
    }
}

/// SID de NT SERVICE\TrustedInstaller, dueño de las fuentes que trae Windows
/// (también las de «características a petición»).
const SID_TRUSTEDINSTALLER: &str = "S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464";

/// ¿El archivo es de TrustedInstaller? None si no se puede leer su propietario.
/// Se recuerda por sesión: la carpeta de fuentes del sistema tiene cientos.
pub fn de_trustedinstaller(ruta: &Path) -> Option<bool> {
    use std::{collections::HashMap, sync::Mutex};
    use windows_sys::Win32::{
        Foundation::{LocalFree, ERROR_SUCCESS},
        Security::{
            Authorization::{ConvertSidToStringSidW, GetNamedSecurityInfoW, SE_FILE_OBJECT},
            OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
        },
    };
    static VISTOS: Mutex<Option<HashMap<String, bool>>> = Mutex::new(None);
    let clave = ruta.to_string_lossy().to_lowercase();
    if let Some(&v) = VISTOS.lock().ok()?.get_or_insert_with(HashMap::new).get(&clave) {
        return Some(v);
    }
    let nombre = ancho(ruta);
    let mut propietario = std::ptr::null_mut();
    let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    // SAFETY: nombre acaba en 0; sd lo reserva Windows y se libera con LocalFree;
    // propietario apunta dentro de sd y no se usa después de liberarlo.
    let es = unsafe {
        let r = GetNamedSecurityInfoW(
            nombre.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            &mut propietario,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut sd,
        );
        if r != ERROR_SUCCESS || propietario.is_null() {
            if !sd.is_null() {
                LocalFree(sd as _);
            }
            return None;
        }
        let mut texto: *mut u16 = std::ptr::null_mut();
        let ok = ConvertSidToStringSidW(propietario, &mut texto) != 0 && !texto.is_null();
        let es = ok && {
            let mut n = 0;
            while *texto.add(n) != 0 {
                n += 1;
            }
            String::from_utf16_lossy(std::slice::from_raw_parts(texto, n)) == SID_TRUSTEDINSTALLER
        };
        if !texto.is_null() {
            LocalFree(texto as _);
        }
        LocalFree(sd as _);
        es
    };
    if let Ok(mut m) = VISTOS.lock() {
        m.get_or_insert_with(HashMap::new).insert(clave, es);
    }
    Some(es)
}

/// Abre una dirección web en el navegador predeterminado.
pub fn abrir_url(url: &str) -> Result<(), String> {
    use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};
    let (verbo, dir) = (ancho("open"), ancho(url));
    // SAFETY: cadenas acabadas en 0 que viven durante la llamada; sin ventana padre.
    let r = unsafe { ShellExecuteW(std::ptr::null_mut(), verbo.as_ptr(), dir.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL) };
    // ShellExecute devuelve un valor mayor que 32 si ha ido bien.
    if r as isize > 32 { Ok(()) } else { Err(format!("no se ha podido abrir {url} ({})", r as isize)) }
}

pub fn ancho(s: impl AsRef<OsStr>) -> Vec<u16> {
    s.as_ref().encode_wide().chain(Some(0)).collect()
}

/// Carga la fuente en la sesión. `privada`: solo para este proceso.
pub fn cargar(ruta: &Path, privada: bool) -> bool {
    let w = ancho(ruta);
    unsafe { AddFontResourceExW(w.as_ptr(), if privada { FR_PRIVATE } else { 0 }, std::ptr::null()) > 0 }
}

/// La descarga de la sesión. Windows cuenta las cargas: se repite hasta que no quede ninguna.
pub fn descargar(ruta: &Path, privada: bool) {
    let w = ancho(ruta);
    for _ in 0..16 {
        if unsafe { RemoveFontResourceExW(w.as_ptr(), if privada { FR_PRIVATE } else { 0 }, std::ptr::null()) } == 0 {
            break;
        }
    }
}

/// WM_FONTCHANGE a todas las ventanas. Con tiempo límite: un programa colgado
/// no deja colgada a tuna-fonts.
pub fn avisar_cambio() {
    unsafe {
        SendMessageTimeoutW(HWND_BROADCAST, WM_FONTCHANGE, 0, 0, SMTO_ABORTIFHUNG, 1000, std::ptr::null_mut());
    }
}

/// Programa borrar (o mover) un archivo en el próximo arranque, para cuando Windows lo tiene en uso.
pub fn al_reiniciar(origen: &Path, destino: Option<&Path>) -> bool {
    let o = ancho(origen);
    let d = destino.map(ancho);
    unsafe { MoveFileExW(o.as_ptr(), d.as_ref().map_or(std::ptr::null(), |d| d.as_ptr()), MOVEFILE_DELAY_UNTIL_REBOOT) != 0 }
}

/// Lanza este mismo .exe con permisos de administrador y espera a que acabe.
/// Devuelve su código de salida, o None si el usuario dijo que no al UAC.
pub fn como_admin(args: &str) -> Option<u32> {
    let exe = std::env::current_exe().ok()?;
    let verbo = ancho("runas");
    let archivo = ancho(&exe);
    let parametros = ancho(args);
    unsafe {
        let mut info: SHELLEXECUTEINFOW = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        info.fMask = SEE_MASK_NOCLOSEPROCESS;
        info.lpVerb = verbo.as_ptr();
        info.lpFile = archivo.as_ptr();
        info.lpParameters = parametros.as_ptr();
        info.nShow = SW_HIDE;
        if ShellExecuteExW(&mut info) == 0 || info.hProcess.is_null() {
            return None;
        }
        let mut codigo = 1u32;
        if WaitForSingleObject(info.hProcess, INFINITE) == WAIT_OBJECT_0 {
            GetExitCodeProcess(info.hProcess, &mut codigo);
        }
        CloseHandle(info.hProcess);
        Some(codigo)
    }
}

/// Familias visibles en la sesión, incluidas las activadas por otro gestor.
pub fn familias_visibles() -> std::collections::HashSet<String> {
    use windows_sys::Win32::Graphics::Gdi::{DEFAULT_CHARSET, EnumFontFamiliesExW, GetDC, ReleaseDC, LOGFONTW, TEXTMETRICW};
    use std::collections::HashSet;
    unsafe extern "system" fn recoger(font: *const LOGFONTW, _: *const TEXTMETRICW, _: u32, data: isize) -> i32 {
        let nombre = unsafe { &(*font).lfFaceName };
        let fin = nombre.iter().position(|&c| c == 0).unwrap_or(nombre.len());
        let familias = unsafe { &mut *(data as *mut HashSet<String>) };
        familias.insert(String::from_utf16_lossy(&nombre[..fin]).to_lowercase());
        1
    }
    let mut familias = HashSet::new();
    unsafe {
        let dc = GetDC(std::ptr::null_mut());
        if !dc.is_null() {
            let mut filtro: LOGFONTW = std::mem::zeroed();
            filtro.lfCharSet = DEFAULT_CHARSET;
            EnumFontFamiliesExW(dc, &filtro, Some(recoger), &mut familias as *mut _ as isize, 0);
            ReleaseDC(std::ptr::null_mut(), dc);
        }
    }
    familias
}

/// La barra de tareas pide ICON_BIG y la ventana pide ICON_SMALL. Ambos salen
/// del grupo de iconos del ejecutable generado por tauri-build (IDI_APPLICATION).
pub fn poner_iconos_ventana(hwnd: windows_sys::Win32::Foundation::HWND) -> std::io::Result<()> {
    use windows_sys::Win32::{
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            GetSystemMetrics, LoadImageW, SendMessageW, ICON_BIG, ICON_SMALL, IDI_APPLICATION,
            IMAGE_ICON, LR_SHARED, SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON, WM_SETICON,
        },
    };
    unsafe {
        let modulo = GetModuleHandleW(std::ptr::null());
        if modulo.is_null() { return Err(std::io::Error::last_os_error()); }
        for (tipo, ancho, alto) in [(ICON_SMALL, SM_CXSMICON, SM_CYSMICON), (ICON_BIG, SM_CXICON, SM_CYICON)] {
            let icono = LoadImageW(modulo, IDI_APPLICATION, IMAGE_ICON, GetSystemMetrics(ancho), GetSystemMetrics(alto), LR_SHARED);
            if icono.is_null() { return Err(std::io::Error::last_os_error()); }
            SendMessageW(hwnd, WM_SETICON, tipo as usize, icono as isize);
        }
    }
    Ok(())
}
