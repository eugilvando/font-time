<script>
  import { llamar, estiloDe } from "../lib/fuentes.svelte.js";
  import { t } from "../lib/i18n.svelte.js";
  import { avisar } from "../lib/avisos.svelte.js";
  import bloquesTxt from "../lib/Blocks.txt?raw";
  let { f, estilo } = $props();
  const bloques = bloquesTxt.split(/\r?\n/).flatMap(line => {
    const m = /^([0-9A-F]+)\.\.([0-9A-F]+); (.+)$/.exec(line);
    return m ? [{min:parseInt(m[1],16),max:parseInt(m[2],16),name:m[3]}] : [];
  });
  const traducciones = {"Basic Latin":"latin","Currency Symbols":"currency","Latin-1 Supplement":"latin1","Private Use Area":"private"};
  const titulo = name => name === "gid" ? t("gl.unencoded") : traducciones[name] ? t("gl."+traducciones[name]) : name;
  const hex = cp => "U+" + cp.toString(16).toUpperCase().padStart(4,"0");
  const PAGE = 64;
  let buscar = $state(""), abierto = $state(null), pagina = $state(0);
  let mapa = $state.raw(null), error = $state(""), trazos = $state.raw({});
  const fuente = $derived(estilo && {familia:f.nombre,archivo:estilo.archivo,nombre:estilo.nombre,peso:estilo.peso,cursiva:estilo.cursiva});
  const css = $derived(estilo && estiloDe(f,estilo));
  $effect(() => {
    const actual=fuente;
    let vivo=true;
    mapa=null; error=""; abierto=null; pagina=0;
    if (actual) llamar("mapa_glifos",{fuente:actual}).then(r => {if(vivo) mapa=r;}).catch(e => {if(vivo) error=String(e);});
    return () => { vivo=false; };
  });
  const grupos = $derived.by(() => {
    if (!mapa) return [];
    const porId=mapa.nombres.map((nombre,id) => ({id,nombre,cps:[],bloque:"gid"}));
    let pos=0;
    for (const [cp,id] of mapa.codigos) {
      while(pos+1<bloques.length && cp>bloques[pos].max) pos++;
      const g=porId[id];
      if (!g) continue;
      if (!g.cps.length) g.bloque=cp>=bloques[pos].min && cp<=bloques[pos].max ? bloques[pos].name : "Unicode";
      g.cps.push(cp);
    }
    porId.sort((a,b)=>(a.cps[0]??Infinity)-(b.cps[0]??Infinity));
    const grupos=new Map();
    for(const glifo of porId) {
      if(!grupos.has(glifo.bloque)) grupos.set(glifo.bloque,[]);
      grupos.get(glifo.bloque).push(glifo);
    }
    return [...grupos];
  });
  const visibles = $derived.by(() => {
    const q=buscar.trim().toLowerCase();
    if(!q) return grupos;
    return grupos.map(([b,gs]) => [b,titulo(b).toLowerCase().includes(q) || b.toLowerCase().includes(q) ? gs : gs.filter(g =>
      g.nombre.toLowerCase().includes(q) || ("gid "+g.id)===q || g.cps.some(c => String.fromCodePoint(c).toLowerCase()===q || hex(c).toLowerCase().includes(q))) ]).filter(([,gs])=>gs.length);
  });
  const actual = $derived(abierto === null ? visibles[0]?.[0] : abierto);
  const lista = $derived(visibles.find(([b]) => b===actual)?.[1] ?? []);
  const ventana = $derived(lista.slice(pagina*PAGE,(pagina+1)*PAGE));
  $effect(() => {
    const target=fuente, ids=ventana.map(g => g.id);
    let vivo=true; trazos={};
    if(target && ids.length) llamar("trazos_glifos",{fuente:target,ids}).then(r=>{if(vivo) trazos=r;}).catch(e=>{if(vivo) error=String(e);});
    return () => {vivo=false;};
  });
  function etiqueta(g) {return [g.nombre,...g.cps.map(hex),"GID "+g.id].filter(Boolean).join(" · ");}
  async function copiar(g) {
    const path=trazos[g.id];
    const texto=g.cps.length ? String.fromCodePoint(g.cps[0]) : path && `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${path[0]}"><path d="${path[1]}"/></svg>`;
    if (!texto) return;
    try { await navigator.clipboard.writeText(texto); avisar("ok",t("ok.copiado",{c:g.cps.length ? hex(g.cps[0]) : "SVG"}),""); }
    catch(e) { avisar("err",t("err.una"),String(e)); }
  }
</script>
<section class="panel" aria-label={t("gl.title")}>
  <header><h2>{t("gl.title")}</h2>

  </header>
  <div class="tools"><span aria-live="polite">{t("gl.count",{n:visibles.reduce((n,[,gs])=>n+gs.length,0),total:mapa?.nombres.length??0})}</span>
    <input type="search" aria-label={t("gl.search")} placeholder={t("gl.search")} bind:value={buscar} oninput={()=>{ abierto=null;pagina=0; }} /></div>
  {#if error}<p role="status">{error}</p>{/if}
  {#each visibles as [b,gs] (b)}
    <button class="block" type="button" aria-expanded={actual===b} onclick={()=>{abierto=actual===b?"":b;pagina=0;}}><span>{actual===b?"▾":"▸"} {titulo(b)}</span><span>{gs.length}</span></button>
    {#if b===actual}
      <div class="grid">{#each ventana as g (g.id)}
        <button class="glyph" title={etiqueta(g)} aria-label={etiqueta(g)} disabled={!g.cps.length && !trazos[g.id]} onclick={()=>copiar(g)}>
          {#if trazos[g.id]}<svg viewBox={trazos[g.id][0]} aria-hidden="true"><path d={trazos[g.id][1]} fill="currentColor"/></svg>
          {:else if g.cps.length}<span style:font-family={css.familia}>{String.fromCodePoint(g.cps[0])}</span>
          {:else}<small>GID {g.id}</small>{/if}
        </button>
      {/each}</div>
      {#if gs.length>PAGE}<nav><button aria-label={t("gl.prev")} disabled={!pagina} onclick={()=>pagina--}>‹</button>{pagina+1} / {Math.ceil(gs.length/PAGE)}<button aria-label={t("gl.next")} disabled={(pagina+1)*PAGE>=gs.length} onclick={()=>pagina++}>›</button></nav>{/if}
    {/if}
  {/each}
</section>
<style>
  .panel {background:var(--field);border:1px solid var(--line2);border-radius:12px;padding:16px 20px;}
  header,.tools,.block,nav {display:flex;align-items:center;gap:12px;justify-content:space-between;flex-wrap:wrap;}
  h2 {margin:0;font:600 20px var(--sans);}
  .tools {margin:14px 0;color:var(--ink2);font-size:13px;}
  input,nav button {border:1px solid var(--line2);border-radius:6px;background:var(--win);color:var(--ink);padding:7px 10px;max-width:100%;}
  .block {border:0;border-top:1px solid var(--line);background:none;width:100%;padding:10px 0;color:var(--ink);font-weight:600;}
  .block span:last-child {color:var(--ink2);font-variant-numeric:tabular-nums;}
  .grid {display:grid;grid-template-columns:repeat(auto-fill,minmax(49px,1fr));gap:3px;margin-bottom:10px;}
  .glyph {display:grid;place-items:center;height:48px;background:var(--win);color:var(--ink);border:0;border-radius:5px;font-size:26px;}
  .glyph svg {height:35px;width:80%;}
  .glyph:hover {background:var(--chip);}
  nav {justify-content:flex-end;padding:7px 0 12px;} nav button {min-width:32px;}
</style>
