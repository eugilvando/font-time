<script>
  import { ChevronDown, X } from "@lucide/svelte";
  import { configurar, etiquetasDe, grupoDe } from "../lib/organizacion.svelte.js";
  import { t } from "../lib/i18n.svelte.js";
  import CampoGrupo from "./CampoGrupo.svelte";
  let { f, soloTags = false } = $props();
  const id = $props.id();
  const tags = $derived(etiquetasDe(f));
  let entrada = $state("");
  let grupo = $derived(grupoDe(f.nombre));
  function agregar() {
    const nuevas = entrada.split(",").map(x => x.trim()).filter(Boolean);
    if (nuevas.length) configurar("etiquetas", f.nombre, [...new Set([...tags.personal, ...nuevas])]);
    entrada = "";
  }
</script>
<section class="panel" aria-label={t("org.organize")}>
  {#if !soloTags}<h2>{t("org.organize")}</h2>{/if}
  <div class="campos">
    {#if !soloTags}<div><CampoGrupo bind:valor={grupo} onguardar={g => configurar("grupos",f.nombre,g)} /></div>{/if}
    <div>
      <label for={id}>{t("org.tagLabel")}</label>
      <input {id} bind:value={entrada} placeholder={t("org.addTag")} onblur={agregar} onkeydown={e => {
        if (!e.isComposing && (e.key === "Enter" || e.key === ",")) { e.preventDefault(); agregar(); }
      }} />
      {#if tags.personal.length}
        <ul>{#each tags.personal as tag (tag)}<li class="personal">{tag}<button type="button" title={t("org.removeTag",{tag})} aria-label={t("org.removeTag",{tag})} onclick={() => configurar("etiquetas",f.nombre,tags.personal.filter(x => x !== tag))}><X size={13} /></button></li>{/each}</ul>
      {/if}
    </div>
  </div>
  {#each [[t("org.attributes"),tags.sistema],[t("org.googleTags"),tags.comunidad]] as [titulo, items] (titulo)}
    {#if items.length}
      <details>
        <summary><ChevronDown size={15}/><span>{titulo}</span><span class="count">{items.length}</span></summary>
        <ul>{#each items as item (item)}<li>{item}</li>{/each}</ul>
      </details>
    {/if}
  {/each}
</section>
<style>
  .panel { border:1px solid var(--line2,var(--line)); border-radius:12px; background:var(--win); padding:18px 20px 8px; color:var(--ink); }
  h2 { font:600 17px/1.4 var(--sans); margin:0 0 17px; }
  .campos { display:grid; grid-template-columns:repeat(auto-fit,minmax(min(100%,220px),1fr)); gap:14px 24px; padding-bottom:16px; }
  label { display:block; font:600 13px var(--sans); margin-bottom:8px; }
  input { box-sizing:border-box; width:100%; min-height:38px; padding:8px 11px; font:400 13px var(--sans); color:var(--ink); background:var(--win); border:1px solid var(--line2,var(--line)); border-radius:8px; }
  input::placeholder { color:var(--ink2); }
  :is(input,summary,button):focus-visible { outline:2px solid var(--ink2); outline-offset:2px; }
  details { border-top:1px solid var(--line); }
  summary { display:flex; align-items:center; gap:8px; padding:10px 0; min-height:34px; box-sizing:border-box; cursor:pointer; color:var(--ink2); font:500 13px var(--sans); list-style:none; }
  summary::-webkit-details-marker { display:none; }
  summary :global(svg) { transform:rotate(-90deg); }
  details[open] summary :global(svg) { transform:none; }
  .count { margin-left:auto; font-variant-numeric:tabular-nums; }
  ul { display:flex; flex-wrap:wrap; gap:6px; list-style:none; margin:8px 0; padding:0; }
  details ul { margin:0 0 14px 22px; }
  li { max-width:100%; overflow-wrap:anywhere; border-radius:6px; background:var(--chip); color:var(--ink2); padding:4px 8px; font:500 12px/1.4 var(--sans); }
  .personal { display:inline-flex; gap:4px; align-items:center; padding-right:3px; color:var(--ink); }
  button { color:inherit; background:transparent; border:0; padding:3px; display:grid; border-radius:4px; }
  button:hover { background:var(--hov); }
</style>