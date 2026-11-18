// Ajustes de la app. Se guardan con tauri-plugin-store en ajustes.json; Rust
// lee ese mismo archivo al arrancar y lo inyecta en window.__TUNA__ (ver
// src-tauri/src/lib.rs), así que el primer fotograma ya sale con el tema y el
// idioma buenos, sin esperar a nada.
import { LazyStore } from "@tauri-apps/plugin-store";
import { version as versionPaquete } from "../../package.json";

// Fuera de Tauri (vista previa en el navegador) la versión sale de package.json.
const inicio = window.__TUNA__ ?? { ajustes: {}, version: versionPaquete };

export const plataforma = inicio.plataforma ?? "windows";
export const esWindows = plataforma === "windows";
const idiomaNavegador = navigator.language?.split("-")[0].toLowerCase();
const idiomaSistema = inicio.idiomaSistema ?? ({ pt: "pt-BR", es: "es" }[idiomaNavegador] ?? "en");
export const enTauri = "__TAURI_INTERNALS__" in window;
export const version = inicio.version;

export const ATAJOS = { buscar: "Ctrl+F", todas: "Ctrl+A", alternar: "Space" };

const POR_DEFECTO = {
  // null = el de Windows la primera vez; luego, el que elijas.
  tema: null,
  // El de Windows la primera vez (es si está en español, en para lo demás); luego, el que elijas.
  idioma: idiomaSistema,
  // true en cuanto se elige el idioma a mano en ajustes (desde la 0.2.1).
  idiomaElegido: false,
  // usuario | todos (pide permisos)
  desactivarAlSalir: false,
  // instaladas | favoritas
  alAbrir: "instaladas",
  carpetaPrincipal: "",
  // bandeja | salir: qué hace la X (en la bandeja, las carpetas vigiladas siguen disponibles)
  alCerrar: "bandeja",
  // Fuentes del sistema desbloqueadas (las imprescindibles, nunca).
  desbloqueadas: false,
  // Ver en la lista las del sistema que llevan candado (por defecto, ocultas).
  verBloqueadas: false,
  // Una consulta a GitHub al abrir para ver si hay versión nueva (desactivable).
  buscarActualizaciones: true,
  // Texto de prueba por defecto ("" = el de siempre del idioma).
  textoPrueba: "",
  atajos: { ...ATAJOS },
  vista: "lista",
  tamano: 42,
};

export const ajustes = $state({ ...POR_DEFECTO, ...inicio.ajustes });
ajustes.atajos = { ...ATAJOS, ...(ajustes.atajos || {}) };

if (!ajustes.idiomaElegido) ajustes.idioma = idiomaSistema;

export function ponerIdioma(idioma) {
  ajustes.idioma = idioma;
  ajustes.idiomaElegido = true;
}

// Tema que se ve ahora mismo (resuelve el null con el de Windows).
const oscuroSistema = matchMedia("(prefers-color-scheme: dark)");
let temaSistema = $state(oscuroSistema.matches ? "dark" : "light");
oscuroSistema.addEventListener("change", (e) => { temaSistema = e.matches ? "dark" : "light"; });
export const temaActual = () => ajustes.tema ?? temaSistema;

const store = enTauri ? new LazyStore("ajustes.json", { autoSave: 150 }) : null;

$effect.root(() => {
  // Al cambiar de tema, sin transiciones durante un par de fotogramas: con
  // miles de casillas en el mapa de caracteres del espécimen, animar el color
  // de todas a la vez atascaba el cambio (y se veía a medias).
  let primera = true;
  $effect(() => {
    const tema = temaActual();
    if (enTauri) import("@tauri-apps/api/window").then(({ getCurrentWindow }) =>
      getCurrentWindow().setBackgroundColor(tema === "dark" ? [0x19, 0x18, 0x16, 255] : [0xf7, 0xf6, 0xf3, 255]));
    const html = document.documentElement;
    if (primera || html.dataset.theme === tema) {
      html.dataset.theme = tema;
      primera = false;
      return;
    }
    html.classList.add("cambiando-tema");
    html.dataset.theme = tema;
    requestAnimationFrame(() => requestAnimationFrame(() => html.classList.remove("cambiando-tema")));
  });
  // Cada ajuste que cambia se guarda; el plugin agrupa las escrituras seguidas.
  for (const clave of Object.keys(POR_DEFECTO)) {
    $effect(() => {
      const valor = $state.snapshot(ajustes[clave]);
      store?.set(clave, valor);
    });
  }
});

export function ponerTema(tema) {
  ajustes.tema = tema;
}
