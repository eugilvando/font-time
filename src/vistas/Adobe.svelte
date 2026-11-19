<script>
  import { ajustes } from "../lib/ajustes.svelte.js";
  import { adobe } from "../lib/adobe.svelte.js";
  import { llamar } from "../lib/fuentes.svelte.js";
  import { t } from "../lib/i18n.svelte.js";
  let mensaje=$state("");
  async function guardar() {
    try { if (await llamar("adobe_script")) mensaje=t("org.adobeInstall"); } catch(e) { mensaje=String(e); }
  }
</script>
<h3>Adobe Creative Cloud · {t("org.experimental")}</h3>
<small>{t("org.adobeHint")}</small>
<label><input type="checkbox" bind:checked={ajustes.adobe} /> {t("org.adobeOpen")}</label>
<label><input type="checkbox" disabled={!ajustes.adobe} bind:checked={ajustes.adobeCerrar} /> {t("org.adobeClose")}</label>
<button onclick={guardar}>{t("org.adobeScript")}</button>
{#if ajustes.adobe}<small role="status">{t(adobe.connected ? "org.adobeReady" : "org.adobeWait")} {adobe.error}</small>{/if}
{#if mensaje}<small role="status">{mensaje}</small>{/if}
<style>h3 {font-size:13px;} label {font-size:12px;} button {justify-self:start;font:inherit;font-size:12px;background:var(--win);color:inherit;border:1px solid var(--line);border-radius:6px;padding:5px 9px;}</style>