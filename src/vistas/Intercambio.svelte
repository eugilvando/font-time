<script>
  import { biblioteca } from "../lib/biblioteca.svelte.js";
  import { exportar, fusionar, leerColecciones } from "../lib/colecciones.js";
  import { sync, configurar, sincronizar } from "../lib/sync.svelte.js";
  import { llamar, fuentes } from "../lib/fuentes.svelte.js";
  import { t } from "../lib/i18n.svelte.js";
  let entrada=$state(null), error=$state(""), resultado=$state("");
  const faltantes=$derived.by(() => {
    const catalogo=new Set(fuentes.lista.map(f => f.nombre.toLowerCase()));
    return entrada ? [...new Set(entrada.flatMap(c=>c.fuentes).filter(f=>!catalogo.has(f.toLowerCase())))] : [];
  });
  async function importar() {
    try { const texto=await llamar("importar_colecciones"); if(texto!=null) entrada=leerColecciones(texto); error=""; resultado=""; }
    catch(e) { error=String(e); entrada=null; }
  }
  async function guardar(archivo) {
    try { const lista=$state.snapshot(biblioteca.colecciones);
      const ok=await llamar("exportar_colecciones",{ contenido:exportar(lista),archivo,familias:[...new Set(lista.flatMap(c=>c.fuentes))] });
      if(ok) resultado=t("org.saved"); error="";
    } catch(e) { error=String(e); }
  }
</script>
<h3>{t("grupo.colecciones")}</h3>
<div>
  <button onclick={importar}>{t("org.import")}</button>
  <button onclick={() => guardar(false)}>{t("org.export")}</button>
  <button onclick={() => guardar(true)}>{t("org.zip")}</button>
</div>
{#if entrada}
  <p>{t("org.review",{n:entrada.length})} · {t("org.missing",{n:faltantes.length})}</p>
  {#if faltantes.length}<details><summary>{t("org.missing",{n:faltantes.length})}</summary>{faltantes.join(", ")}</details>{/if}
  <div><button onclick={() => {biblioteca.colecciones=fusionar(biblioteca.colecciones,entrada);entrada=null;resultado=t("org.saved");}}>{t("org.merge")}</button>
  <button onclick={() => entrada=null}>{t("org.cancel")}</button></div>
{/if}
{#if resultado || error}<small role="status">{error || resultado}</small>{/if}
<h3>{t("org.sync")}</h3>
<small>{t("org.syncHint")}</small>
<div><button disabled={sync.ocupado} onclick={() => configurar("pick")}>{t("org.pick")}</button>
{#if sync.carpeta}<button disabled={sync.ocupado} onclick={sincronizar}>{t("org.syncNow")}</button>
<button disabled={sync.ocupado} onclick={() => configurar("off")}>{t("org.disconnect")}</button>{/if}</div>
{#if sync.carpeta}<small>{sync.carpeta}</small>{/if}
{#if sync.error}<small role="alert">{sync.error}</small>{/if}
<style>h3 {font-size:13px;} div {display:flex;gap:6px;flex-wrap:wrap;} button {font:inherit;font-size:12px;background:var(--win);color:inherit;border:1px solid var(--line);border-radius:6px;padding:5px 9px;} small {overflow-wrap:anywhere;}</style>