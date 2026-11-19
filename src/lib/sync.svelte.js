import { biblioteca, guardarMarcaSync, leerMarcaSync } from "./biblioteca.svelte.js";
import { cambios, reproducir, diarioCompleto } from "./colecciones.js";
import { llamar } from "./fuentes.svelte.js";
import { t } from "./i18n.svelte.js";
import { enTauri } from "./ajustes.svelte.js";

export const sync = $state({ carpeta:null, ocupado:false, error:"" });
const copia = () => $state.snapshot(biblioteca.colecciones);
export async function configurar(accion) {
  if (sync.ocupado || !enTauri) return;
  try {
    sync.carpeta = await llamar("configurar_sync",{accion});
    sync.error = "";
    if (sync.carpeta) await sincronizar();
  } catch (e) { sync.error=String(e); }
}
export async function sincronizar() {
  if (!sync.carpeta || !biblioteca.listo || sync.ocupado) return;
  sync.ocupado = true;
  const carpeta = sync.carpeta;
  try {
    const marca = await leerMarcaSync();
    const diario = await llamar("diario_sync",{carpeta,guardar:null});
    const confirmados = marca?.carpeta === carpeta ? marca.lotes ?? [] : [];
    if (!diarioCompleto(diario, confirmados)) {
      sync.error = t("org.pendingSync");
      return;
    }
    const lotes = diario.map(d => d.id);
    const local = copia();
    const base = marca?.carpeta === carpeta ? marca.lista : [];
    const ops = cambios(base,local);
    const remoto = reproducir(diario);
    let lista = remoto.lista;
    if (ops.length) {
      const lote = { version:1, id:crypto.randomUUID(), clock:remoto.clock+1, ops };
      await llamar("diario_sync",{carpeta,guardar:lote});
      lotes.push(lote.id);
      lista = reproducir([...diario,lote]).lista;
    }
    // Rebase edits performed while writing to OneDrive before touching the UI.
    const posteriores = cambios(local,copia());
    let final = lista;
    if (posteriores.length) {
      final = reproducir([{version:1,id:"base",clock:1,ops:cambios([],lista)},
        {version:1,id:"pending",clock:2,ops:posteriores}]).lista;
    }
    if (JSON.stringify(copia()) !== JSON.stringify(final)) biblioteca.colecciones = final;
    if (marca?.carpeta !== carpeta || confirmados.length !== lotes.length || JSON.stringify(base) !== JSON.stringify(lista)) {
      await guardarMarcaSync({carpeta,lista,lotes});
    }
    sync.error="";
  } catch (e) { sync.error=String(e); }
  finally {sync.ocupado=false;}
}
export function iniciarSync() {
  if (!enTauri) return;
  configurar("get");
  const timer = setInterval(sincronizar,20000);
  return () => clearInterval(timer);
}
