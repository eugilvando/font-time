<script>
  import { ajustes } from "../lib/ajustes.svelte.js";
  import { t } from "../lib/i18n.svelte.js";
  let { valor = $bindable(""), onguardar = () => {} } = $props();
  const id = $props.id();
  const grupos = $derived([...new Set(ajustes.grupos.map(g => g.valor))].sort((a,b) => a.localeCompare(b)));
</script>
<label for={id}>{t("org.familyGroup")}</label>
<input {id} list={id + "-list"} bind:value={valor} placeholder={t("org.chooseGroup")} onchange={() => onguardar(valor.trim())} onkeydown={e => { if (e.key === "Enter" && !e.currentTarget.form) e.currentTarget.blur(); }} />
<datalist id={id + "-list"}>{#each grupos as grupo (grupo)}<option value={grupo}></option>{/each}</datalist>
<style>
  label { display:block; font:600 13px var(--sans); color:var(--ink); margin-bottom:8px; }
  input { width:100%; min-width:0; box-sizing:border-box; min-height:38px; padding:8px 11px; border:1px solid var(--line2,var(--line)); border-radius:8px; background:var(--win); color:var(--ink); font:400 13px var(--sans); }
  input::placeholder { color:var(--ink2); }
  input:focus-visible { outline:2px solid var(--ink2); outline-offset:2px; }
</style>