<script>
  import { SlidersHorizontal, X } from "@lucide/svelte";
  import { t } from "../lib/i18n.svelte.js";
  import { ajustes } from "../lib/ajustes.svelte.js";
  import { vacios, cantidad, categorias, medidas, distribucion } from "../lib/filtros.js";
  let { filtros=$bindable(), estilos, busqueda=$bindable(), estadoFijo=false }=$props();
  const id=$props.id();
  let boton,panel, nombre=$state("");
  const n=$derived(cantidad(filtros));
  const datos=$derived(Object.fromEntries(medidas.map(k=>[k,distribucion(estilos,k)])));
  function posicion() {
    if(!panel?.matches(":popover-open")) return;
    const r=boton.getBoundingClientRect(), z=ajustes.escala/100;
    panel.style.left=Math.max(4,Math.min(r.left,innerWidth-panel.offsetWidth*z-6))/z+"px";
    panel.style.top=(r.bottom+4)/z+"px";
    panel.style.maxHeight=Math.max(100,(innerHeight-r.bottom-12)/z)+"px";
  }
  function categoria(c) {
    filtros.categorias=filtros.categorias.includes(c)?filtros.categorias.filter(x=>x!==c):[...filtros.categorias,c];
  }
  function rango(k,i,num) {
    const d=datos[k], par=[...(filtros.rangos[k]||[d.min,d.max])];
    par[i]=i===0?Math.min(num,par[1]):Math.max(num,par[0]);
    if(par[0]===d.min && par[1]===d.max) delete filtros.rangos[k];else filtros.rangos[k]=par;
  }
  function guardar() {
    const name=nombre.trim();
    if(!name)return;
    const nuevo={nombre:name, filtros:$state.snapshot(filtros),busqueda};
    ajustes.filtrosGuardados=[...ajustes.filtrosGuardados.filter(g=>g.nombre!==name),nuevo];
    nombre="";
  }
</script>
<svelte:window onresize={posicion}/>
<button class="trigger" bind:this={boton} popovertarget={id} aria-label={t("filter.title")} title={t("filter.title")}><SlidersHorizontal size={16}/>{#if n}<b>{n}</b>{/if}</button>
<div class="panel" bind:this={panel} {id} popover="auto" ontoggle={posicion} role="region" aria-label={t("filter.title")}>
  <header><strong>{t("filter.title")}</strong><button popovertarget={id} popovertargetaction="hide" aria-label={t("close")}><X size={14}/></button></header>
  <div class="chips">
    {#each ["favoritas","variable","color","duplicada"] as k}
      <button aria-pressed={filtros[k]} onclick={()=>filtros[k]=!filtros[k]}>{t("filter."+k)}</button>
    {/each}
    {#each ["activas","inactivas"] as k}<button disabled={estadoFijo} aria-pressed={filtros.estado===k} onclick={()=>filtros.estado=filtros.estado===k?"todas":k}>{t("vista."+k)}</button>{/each}
  </div>
  <h3>{t("grupo.categorias")}</h3>
  <div class="chips">{#each categorias as c}<button aria-pressed={filtros.categorias.includes(c)} onclick={()=>categoria(c)}>{t("cat."+c)}</button>{/each}</div>
  <input class="tag-input" type="search" aria-label={t("filter.tag")} placeholder={t("filter.tag")} bind:value={filtros.etiqueta}/>
  {#if filtros.etiqueta}<div class="chips"><button aria-pressed="true" onclick={()=>filtros.etiqueta=""}>{filtros.etiqueta} ×</button></div>{/if}
  <h3 title={t("filter.explain")}>{t("filter.properties")}</h3>
  <div class="props">{#each medidas as k}
    {@const d=datos[k]}{@const par=filtros.rangos[k]||[d.min,d.max]}
    <fieldset><legend>{t("filter."+k)}</legend>
      <div class="bars" aria-hidden="true">{#each d.bins as count,i}<i class:inRange={!!count && d.min+(i+1)*(d.max-d.min)/24>=par[0] && d.min+i*(d.max-d.min)/24<=par[1]} style:height={(count?Math.max(6,100*count/Math.max(1,...d.bins)):2)+"%"}></i>{/each}</div>
      {#if d.count}
        {#each par as v,i}<label><span>{t(i?"filter.max":"filter.min")}</span><input type="range" min={d.min} max={d.max} value={v} aria-label={t("filter."+k)+" "+t(i?"filter.max":"filter.min")} oninput={e=>rango(k,i,e.currentTarget.valueAsNumber)}/><output>{v}</output></label>{/each}
      {:else}<small>{t("filter.noData")}</small>{/if}
    </fieldset>
  {/each}</div>
  <footer><button onclick={()=>{const anterior=filtros.estado;filtros=vacios();if(estadoFijo)filtros.estado=anterior;}}>{t("filter.clear")}</button>
    <form onsubmit={e=>{e.preventDefault();guardar();}}><input bind:value={nombre} aria-label={t("filter.name")} placeholder={t("filter.name")} maxlength="80"/><button disabled={!nombre.trim()}>{t("org.save")}</button></form>
  </footer>
</div>
<style>
  .trigger,header button {display:inline-flex;align-items:center;justify-content:center;gap:3px;border:1px solid var(--line2);border-radius:6px;background:var(--field);color:var(--ink);min-height:30px;padding:5px 8px;}
  .trigger b {font-size:10px;}
  .panel {position:fixed;inset:auto;margin:0;padding:14px;width:min(430px,84%);overflow:auto;background:var(--field);color:var(--ink);border:1px solid var(--line2);border-radius:12px;box-shadow:var(--flotante);}
  header,footer,form {display:flex;align-items:center;gap:6px;justify-content:space-between;flex-wrap:wrap;}
  header strong {font-size:16px;} h3 {font-size:13px;margin:14px 0 6px;}
  .tag-input {margin-top:8px;padding:5px 8px;border:1px solid var(--line2);border-radius:6px;background:var(--win);}
  .chips {display:flex;gap:5px;flex-wrap:wrap;margin-top:7px;}
  .chips button,footer button,form input {padding:5px 8px;border-radius:6px;background:var(--win);border:1px solid var(--line2);color:var(--ink);font-size:12px;}
  .chips button[aria-pressed=true]{background:var(--ink);color:var(--win);}
  .props {display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:8px;}
  fieldset {min-width:0;border:1px solid var(--line2);border-radius:6px;padding:6px;}
  legend {font-size:12px;font-weight:600;}
  .bars {display:flex;align-items:flex-end;height:28px;gap:1px;}
  i {flex:1;background:var(--line2);} i.inRange {background:var(--ink2);}
  label {display:flex;align-items:center;gap:3px;font-size:10px;color:var(--ink2);} label input {min-width:0;width:100%;accent-color:var(--ink);} output {width:35px;text-align:right;}
  footer {border-top:1px solid var(--line);margin-top:12px;padding-top:8px;} form input {width:130px;}
</style>
