<script>
  import { tick } from "svelte";
  import { t } from "../lib/i18n.svelte.js";
  import { ajustes } from "../lib/ajustes.svelte.js";
  import { llamar, actualizarFuentes } from "../lib/fuentes.svelte.js";
  import { configurar, grupoDe } from "../lib/organizacion.svelte.js";
  import { avisar } from "../lib/avisos.svelte.js";
  import CampoGrupo from "./CampoGrupo.svelte";
  import Organizacion from "./Organizacion.svelte";
  let { lista, seleccion, bloqueada, onalternar, onabrir } = $props();
  let menu, dialogo;
  let filas=$state.raw([]), foco=$state(null), modo=$state(""), grupo=$state(""), tagFamilia=$state("");
  let rutas=$state.raw([]), ocupado=$state(false);
  const familias=$derived([...new Set(filas.map(f=>f.nombre))]);
  const editables=$derived(filas.filter(f=>!bloqueada(f)));
  const encender=$derived(editables.filter(f=>!f.activa));
  const apagar=$derived(editables.filter(f=>f.activa));
  const clave=f=>f.claveVista||f.nombre;
  const args=()=>({familias:filas.filter(f=>!f.estiloVista).map(f=>f.nombre),estilos:filas.filter(f=>f.estiloVista).map(f=>({
    familia:f.nombre,nombre:f.estiloVista.nombre,peso:f.estiloVista.peso,cursiva:f.estiloVista.cursiva
  }))});
  async function abrir(e) {
    if(dialogo.open || ocupado) return;
    const row=e.target.closest?.("[data-fila]");
    const f=row && lista.find(x=>!x.google && !x.grupo && clave(x)===row.dataset.fila);
    if(!f) return;
    e.preventDefault();
    foco=f; filas=seleccion.some(x=>clave(x)===clave(f))?[...seleccion]:[f];
    menu.showPopover();
    await tick();
    const z=(ajustes.escala || 100)/100, rect=menu.getBoundingClientRect();
    const punto=row.getBoundingClientRect();
    const x=e.clientX || punto.left, y=e.clientY || punto.bottom;
    menu.style.left=Math.max(2,Math.min(x,innerWidth-rect.width-4))/z+"px";
    menu.style.top=Math.max(2,Math.min(y,innerHeight-rect.height-4))/z+"px";
    menu.querySelector("button:not(:disabled)")?.focus();
  }
  function teclado(e) {
    if(!menu.matches(":popover-open")) return;
    if(["ArrowDown","ArrowUp","Home","End"].includes(e.key)){
      e.preventDefault();
      const items=[...menu.querySelectorAll("button:not(:disabled)")], i=items.indexOf(document.activeElement);
      const n=e.key==="Home"?0:e.key==="End"?items.length-1:(i+(e.key==="ArrowDown"?1:-1)+items.length)%items.length;
      items[n]?.focus();
    }
  }
  function alternar(items, valor){ menu.hidePopover(); onalternar(items,valor); }
  function gestionar(accion) {
    menu.hidePopover(); modo=accion; tagFamilia=foco.nombre;
    grupo=familias.every(n=>grupoDe(n)===grupoDe(foco.nombre))?grupoDe(foco.nombre):"";
    dialogo.showModal();
  }
  async function prepararBorrado() {
    gestionar("trash"); rutas=[];ocupado=true;
    try {rutas=(await llamar("papelera",{...args(),confirmadas:null})).rutas;}
    catch(e){avisar("err",t("err.una"),String(e));dialogo.close();}
    finally{ocupado=false;}
  }
  async function borrar() {
    if(ocupado || !rutas.length) return;
    ocupado=true;
    try {
      const r=await llamar("papelera",{...args(),confirmadas:rutas});
      await actualizarFuentes();dialogo.close();
      if(r.errores.length) avisar("err",t("err.una"),r.errores.join("\n"));
      else avisar("ok",t("ctx.trashed",{n:r.rutas.length}),"");
    } catch(e){avisar("err",t("err.una"),String(e));}
    finally{ocupado=false;}
  }
</script>
<svelte:window oncontextmenu={abrir} onkeydown={teclado}/>
<div class="menu" popover="auto" bind:this={menu} role="menu" aria-label={t("ctx.actions")}>
  <button role="menuitem" disabled={!encender.length} onclick={()=>alternar(encender,true)}>{t("ctx.activate",{n:encender.length})}</button>
  <button role="menuitem" disabled={!apagar.length} onclick={()=>alternar(apagar,false)}>{t("ctx.deactivate",{n:apagar.length})}</button>
  <hr/>
  <button role="menuitem" onclick={()=>{menu.hidePopover();onabrir(foco);}}>{t("ctx.view")}</button>
  <button role="menuitem" onclick={()=>gestionar("tags")}>{t("ctx.tags")}</button>
  <button role="menuitem" onclick={()=>gestionar("group")}>{t("org.editGroup")}</button>
  <hr/>
  <button role="menuitem" class="danger" onclick={prepararBorrado}>{t("ctx.delete")}</button>
</div>
<dialog bind:this={dialogo} aria-labelledby="ctx-heading" oncancel={e=>{if(ocupado)e.preventDefault();}}>
  <h2 id="ctx-heading">{t(modo==="group"?"org.editGroup":modo==="tags"?"ctx.tags":"ctx.delete")}</h2>
  {#if modo==="group"}
    <p>{familias.join(" · ")}</p>
    <form onsubmit={e=>{e.preventDefault();for(const n of familias) configurar("grupos",n,grupo.trim());dialogo.close();}}>
      <CampoGrupo bind:valor={grupo}/>
      <footer><button type="button" onclick={()=>dialogo.close()}>{t("org.cancel")}</button><button class="primary">{t("org.save")}</button></footer>
    </form>
  {:else if modo==="tags"}
    <select bind:value={tagFamilia} aria-label={t("ctx.view")}>{#each familias as n}<option value={n}>{n}</option>{/each}</select>
    {#if foco}{#key tagFamilia}<Organizacion f={filas.find(f=>f.nombre===tagFamilia) || foco} soloTags/>{/key}{/if}
    <footer><button onclick={()=>dialogo.close()}>{t("close")}</button></footer>
  {:else}
    <p>{ocupado?t("ctx.loading"):rutas.length?t("ctx.confirm",{n:rutas.length}):t("ctx.none")}</p>
    {#if rutas.length}<ul>{#each rutas as p}<li>{p}</li>{/each}</ul>{/if}
    <footer><button disabled={ocupado} onclick={()=>dialogo.close()}>{t("org.cancel")}</button><button class="danger" disabled={ocupado || !rutas.length} onclick={borrar}>{t("ctx.trash")}</button></footer>
  {/if}
</dialog>
<style>
  .menu,dialog {color:var(--ink);background:var(--field);border:1px solid var(--line2);border-radius:9px;box-shadow:var(--flotante);}
  .menu {position:fixed;inset:auto;margin:0;padding:5px;min-width:194px;}
  .menu button {display:block;width:100%;text-align:left;background:transparent;border:0;border-radius:4px;padding:7px 10px;}
  .menu button:enabled:hover,.menu button:focus-visible {background:var(--chip);}
  button:disabled {color:var(--ink2);opacity:.65;cursor:default;}
  .danger:enabled {color:var(--red);}
  hr {border:0;border-top:1px solid var(--line2);margin:4px;}
  dialog {padding:20px;width:min(450px,85%);max-height:82%;overflow:auto;}
  dialog::backdrop {background:var(--velo);}
  h2 {font-size:18px;margin:0 0 12px;} p,li {overflow-wrap:anywhere;}
  ul {max-height:32vh;overflow:auto;user-select:text;margin:10px 0;padding-left:18px;}
  footer {display:flex;justify-content:flex-end;gap:8px;margin-top:14px;}
  footer button,select {border-radius:6px;border:1px solid var(--line2);background:var(--win);color:var(--ink);padding:6px 10px;}
  .primary {background:var(--ink);color:var(--win);} select {margin-bottom:10px;}
</style>
