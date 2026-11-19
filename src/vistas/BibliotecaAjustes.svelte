<script>
  import { ajustes, esWindows, plataforma } from "../lib/ajustes.svelte.js";
  import { actualizadorDisponible } from "../lib/actualizador.svelte.js";
  import { muestrasActuales } from "../lib/organizacion.svelte.js";
  import { t } from "../lib/i18n.svelte.js";
  import Intercambio from "./Intercambio.svelte";
  import Adobe from "./Adobe.svelte";
  let borrador = $state(null);
  function guardarMuestras() {
    if (borrador !== null) ajustes.muestras = [...new Set(borrador.split(/\r?\n/).map(x => x.trim()).filter(Boolean))];
    borrador = null;
  }
</script>

<section>
  <h3>{t("org.display")}</h3>
  <label>{t("tamano")} <input type="range" min="10" max="96" bind:value={ajustes.tamano} /> {ajustes.tamano}</label>
  <label>{t("org.scroll")} <select bind:value={ajustes.velocidad}><option value={0.5}>{t("org.slow")}</option><option value={1}>Normal</option><option value={2}>{t("org.fast")}</option></select></label>
  <label><input type="checkbox" bind:checked={ajustes.agrupar} /> {t("org.group")}</label>
  {#if ajustes.grupos.length}<button onclick={() => ajustes.grupos = []}>{t("org.resetGroups")}</button>{/if}
  <label class="vertical">{t("org.samples")}<textarea rows="5" value={borrador ?? muestrasActuales().join("\n")} oninput={e => borrador = e.currentTarget.value} onblur={guardarMuestras}></textarea></label>
  <h3>{t("org.tags")}</h3>
  <label><input type="checkbox" bind:checked={ajustes.tagsSistema} /> {t("org.systemTags")}</label>
  <label><input type="checkbox" bind:checked={ajustes.tagsPublicos} /> {t("org.communityTags")}</label>
  {#if esWindows}
    <label><input type="checkbox" bind:checked={ajustes.inicioOculto} /> {t("org.hidden")}</label>
  {/if}
  {#if actualizadorDisponible && esWindows}
    <label><input type="checkbox" bind:checked={ajustes.autoInstalar} /> {t("org.autoUpdate")}</label>
  {/if}
  <Intercambio />
  {#if esWindows || plataforma === "macos"}<Adobe />{/if}
</section>
<style>
  section { display:grid; gap:12px; margin:16px 0; }
  h3 { font-size:13px; margin:10px 0 0; }
  label { display:flex; align-items:center; gap:8px; font-size:12px; }
  .vertical { align-items:stretch; flex-direction:column; }
  textarea, select, button { background:var(--win); color:var(--ink); border:1px solid var(--line); border-radius:6px; padding:6px 8px; font:inherit; }
  button { justify-self:start; }
</style>