<script>
  import { llamar } from "../lib/fuentes.svelte.js";
  import { t } from "../lib/i18n.svelte.js";
  let { f, muestra, estilo } = $props();
  let ejes = $state([]);
  let valores = $state({});
  $effect(() => {
    let actual = true;
    ejes = []; valores = {};
    if (!f.error) llamar("ejes_fuente", { ruta:f.archivo, familia:f.nombre }).then(r => { if(actual) ejes=r; }).catch(() => {});
    return () => { actual=false; };
  });
  const css = $derived(ejes.map(e => '"'+e.tag+'" '+(valores[e.tag] ?? e.value)).join(", "));
</script>
{#if ejes.length}
<section>
  <strong>{t("org.axes")}</strong>
  {#each ejes as e (e.tag)}
    <label>{e.label} <input type="range" min={e.min} max={e.max} step="any" value={valores[e.tag] ?? e.value} oninput={ev => valores[e.tag]=ev.currentTarget.valueAsNumber} /> {Math.round(valores[e.tag] ?? e.value)}</label>
  {/each}
  <p style:font-family={estilo.familia} style:font-variation-settings={css}>{muestra}</p>
</section>
{/if}
<style>section { display:grid;gap:8px; } label { font-size:12px; } p { font-size:36px; }</style>