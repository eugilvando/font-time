<script>
  // Espécimen: escala de tamaños, juegos de caracteres, el mapa de caracteres
  // real de la fuente (tabla cmap; clic para copiar), los estilos y la ruta.
  import { ChevronLeft, FolderOpen } from "@lucide/svelte";
  import { ajustes } from "../lib/ajustes.svelte.js";
  import Organizacion from "../componentes/Organizacion.svelte";
  import { adobe, mantener } from "../lib/adobe.svelte.js";
  import Glifos from "./Glifos.svelte";
  import Ejes from "./Ejes.svelte";
  import Corazon from "../componentes/Corazon.svelte";
  import { estiloDe, mostrarEnExplorador, llamar } from "../lib/fuentes.svelte.js";
  import { t } from "../lib/i18n.svelte.js";

  let { f, inicial, muestra, fav, bloqueada, volver, onvolver, onalternar, onestilo, onfav } = $props();

  const tabs=["styles","licenses","waterfall","glyphs","details"];
  let tab=$state("styles");
  const id=$props.id();
  let elegida=$derived(Math.max(0,f.estilos.findIndex(s=>s.nombre===inicial?.nombre && s.peso===inicial?.peso && s.cursiva===inicial?.cursiva)));
  const estilo=$derived(f.estilos[elegida] || f.estilos[0]);
  const e = $derived(estiloDe(f,estilo));
  let licencia=$state(null), fallo=$state("");
  $effect(()=>{
    let vivo=true;licencia=null;fallo="";
    if(tab==="licenses" && estilo) llamar("licencia_fuente",{fuente:{...estilo,familia:f.nombre}}).then(v=>{if(vivo) licencia=v;}).catch(e=>{if(vivo) fallo=String(e);});
    return ()=>{vivo=false;};
  });
  function moverTab(event,i) {
    const tecla=event.key;
    if(!["ArrowLeft","ArrowRight","Home","End"].includes(tecla)) return;
    event.preventDefault();
    const n=tecla==="Home"?0:tecla==="End"?tabs.length-1:(i+(tecla==="ArrowRight"?1:-1)+tabs.length)%tabs.length;
    tab=tabs[n];document.getElementById(id+"-"+tab)?.focus();
  }
  const otros = $derived(f.categoria === "otros");
  const meta = $derived(`${f.estilos.length} ${f.estilos.length === 1 ? t("estilo") : t("estilos")} · ${t("origen." + f.origen)}`);
  const juegos = $derived(
    otros
      ? [{ titulo: t("muestra.titulo"), chars: muestra }, { titulo: t("numeros.solo"), chars: "0123456789" }]
      : [
          { titulo: t("mayusculas"), chars: "ABCDEFGHIJKLMNÑOPQRSTUVWXYZ" },
          { titulo: t("minusculas"), chars: "abcdefghijklmnñopqrstuvwxyz" },
          { titulo: t("numeros"), chars: "0123456789 ¿?¡!@#€%&()«»" },
        ],
  );

</script>

<div class="esp">
  <button type="button" class="volver" onclick={onvolver}><ChevronLeft size={13} strokeWidth={2.2} />{volver}</button>

  <div class="cab">
    <div class="tit">
      <span class="meta">{meta}</span>
      <span class="nombre" style:font-family={e.familia} style:font-weight={e.peso} style:font-style={e.cursiva ? "italic" : "normal"}>{f.nombre}</span>
    </div>
    <button type="button" class="chip" onclick={onfav}>
      <span class="cor" class:fav><Corazon lleno={fav} tam={13} /></span>{fav ? t("fav.es") : t("fav.poner")}
    </button>
    {#if bloqueada}
      <span class="chip fijo">{t("alwaysOn")}</span>
    {:else}
      <button type="button" class="chip" class:principal={!f.activa} onclick={onalternar}>{t(f.activa?"sheet.deactivateFamily":"sheet.activateFamily")}</button>
    {/if}
  </div>

  {#if ajustes.adobe && adobe.owned.includes(f.nombre)}<button class="chip" onclick={() => mantener(f.nombre)}>{t("org.pin")}</button>{/if}
  {#if f.estilos.length}
    <label class="select-style"><span>{t("estilo")}</span><select bind:value={elegida}>{#each f.estilos as s,i}<option value={i}>{s.nombre}</option>{/each}</select></label>
  {/if}
  <div class="tabs" role="tablist" aria-label={f.nombre}>
    {#each tabs as nombre,i}
      <button role="tab" id={id+"-"+nombre} aria-selected={tab===nombre} aria-controls={id+"-panel"} tabindex={tab===nombre?0:-1} onclick={()=>tab=nombre} onkeydown={ev=>moverTab(ev,i)}>{t("sheet."+nombre)}</button>
    {/each}
  </div>
  <div class="contenido" role="tabpanel" id={id+"-panel"} aria-labelledby={id+"-"+tab}>
  {#if tab==="styles"}
  <div class="bloque">
    <span class="etiqueta">{t("estilos.titulo", { n: f.estilos.length })}</span>
    {#each f.estilos as s (s.nombre + s.peso + s.cursiva)}
      <div class="estilo">
        <div class="style-info"><span>{s.nombre}</span>
          <button type="button" class="style-toggle" class:active={s.activa} disabled={!s.editable} aria-pressed={!!s.activa}
            aria-label={t(s.activa ? "deactivate" : "activate") + " · " + s.nombre}
            title={!s.editable ? t("org.styleLock") : s.nombre} onclick={() => onestilo(s)}>
            <i></i>{t(s.activa ? "deactivate" : "activate")}
          </button>
        </div>
        <span class="linea" style:font-family={estiloDe(f, s).familia} style:font-weight={estiloDe(f, s).peso} style:font-style={estiloDe(f, s).cursiva ? "italic" : "normal"} style:font-size="26px">{muestra}</span>
      </div>
    {/each}
  </div>

  {:else if tab==="licenses"}
    {#if fallo}<p role="status">{fallo}</p>{:else if licencia}
      {#if licencia.terms}<p class="license">{licencia.terms}</p>{/if}
      {#if licencia.url}<p class="license">{licencia.url}</p>{/if}
      {#if !licencia.terms && !licencia.url}<p>{t("sheet.noLicense")}</p>{/if}
      {#if licencia.copyright}<p class="license">{licencia.copyright}</p>{/if}
    {:else if !estilo}<p>{t("sheet.noLicense")}</p>{/if}
  {:else if tab==="waterfall"}
  <Ejes f={{...f,archivo:estilo?.archivo??f.archivo}} {muestra} estilo={e} />
  <div class="escala">
    {#each [12, 18, 24, 36, 48, 72] as n (n)}
      <div class="paso">
        <span class="n num">{n}</span>
        <span class="linea" style:font-family={e.familia} style:font-weight={e.peso} style:font-style={e.cursiva ? "italic" : "normal"} style:font-size="{n}px">{muestra}</span>
      </div>
    {/each}
  </div>

  <div class="juegos">
    {#each juegos as j (j.titulo)}
      <div class="juego">
        <span class="etiqueta">{j.titulo}</span>
        <span class="chars" style:font-family={e.familia} style:font-weight={e.peso} style:font-style={e.cursiva ? "italic" : "normal"}>{j.chars}</span>
      </div>
    {/each}
  </div>

  {:else if tab==="glyphs"}
    <Glifos {f} {estilo}/>
  {:else}
    <Organizacion {f}/>
    {#if estilo?.identicos?.length}<details><summary>{t("filter.fileMatches")} · {estilo.identicos.length}</summary>{#each estilo.identicos as p}<button class="archivo" onclick={()=>mostrarEnExplorador(p)}>{p}</button>{/each}</details>{/if}
    {#if estilo}<strong>{estilo.nombre} · {t("filter.peso")} {estilo.peso}</strong><span>{t("sheet.styleFiles")}</span>{/if}
  <div class="ruta">
    {#each (estilo?.fuentes?.length ? estilo.fuentes : estilo ? [estilo.archivo] : f.archivos) as a (a)}
      <button type="button" class="archivo" title={t("ruta.mostrar")} onclick={() => mostrarEnExplorador(a)}>
        <span class="txt">{a}</span>
        <FolderOpen size={13} strokeWidth={1.9} />
      </button>
    {/each}
  </div>
  {/if}
  </div>
</div>

<style>
  .tabs {display:flex;gap:12px;border-bottom:1px solid var(--line2);overflow-x:auto;}
  .tabs button {flex:none;border:0;border-bottom:2px solid transparent;padding:8px 3px;background:none;color:var(--ink2);font-weight:600;}
  .tabs button[aria-selected=true] {border-color:var(--ink);color:var(--ink);}
  .contenido {display:grid;gap:22px;min-width:0;}
  .select-style {display:flex;gap:8px;align-items:center;font-weight:600;}
  .select-style select {background:var(--field);border:1px solid var(--line2);border-radius:6px;padding:5px 8px;color:var(--ink);}
  .license {white-space:pre-wrap;overflow-wrap:anywhere;user-select:text;margin:0;}
  .esp { flex: 1; min-height: 0; overflow-y: auto; padding: 4px 40px 70px 24px; display: flex; flex-direction: column; gap: 18px; animation: tf-up 0.25s ease-out; }
  .esp > :global(*) { flex-shrink: 0; }
  .volver, .chip {
    height: 28px;
    padding: 0 12px 0 8px;
    border-radius: 999px;
    border: 0;
    background: var(--chip);
    color: var(--ink);
    font: 600 12.5px var(--sans);
    display: flex;
    align-items: center;
    gap: 4px;
    flex: none;
  }
  .volver { align-self: flex-start; }
  .chip { height: 30px; padding: 0 12px; gap: 6px; }
  .chip.principal { background: var(--ink); color: var(--win); padding: 0 14px; }
  .chip.fijo { color: var(--ink2); }
  .cor { display: flex; }
  .cor.fav { color: var(--red); }
  .cab { display: flex; align-items: flex-end; gap: 16px; }
  .tit { display: flex; flex-direction: column; gap: 6px; flex: 1; min-width: 0; }
  .meta { font-size: 12px; color: var(--ink3); }
  /* El título no se recorta: las caligráficas tienen adornos que suben y bajan
     mucho más que la línea. Interlineado amplio, sin overflow, y si no cabe a lo
     ancho pasa a la línea siguiente. */
  .nombre { font-size: 64px; line-height: 1.6; padding-top: 0.1em; overflow-wrap: anywhere; }
  .escala { display: flex; flex-direction: column; }
  .paso { display: grid; grid-template-columns: 40px minmax(0, 1fr); align-items: baseline; gap: 12px; padding: 4px 0; }
  .paso .n { font-size: 11px; color: var(--ink3); }
  /* Recorta solo a lo ancho (para los «…»): por arriba y por abajo los adornos de las caligráficas pueden salirse. */
  .linea { line-height: 1.38; white-space: nowrap; overflow-x: clip; overflow-y: visible; text-overflow: ellipsis; }
  /* La escala no se corta por la derecha: lo que no cabe pasa a la línea siguiente. */
  .escala .linea { white-space: normal; overflow: visible; overflow-wrap: anywhere; line-height: 1.3; padding: 0.12em 0; margin: 0; }
  .juegos { display: flex; flex-direction: column; gap: 18px; }
  .juego { display: flex; flex-direction: column; gap: 4px; }
  .chars { font-size: 30px; line-height: 1.5; letter-spacing: 0.02em; word-break: break-all; }
  .bloque { display: flex; flex-direction: column; gap: 8px; }
  .estilo { display: flex; flex-direction: column; gap: 1px; padding: 6px 0; }
  .style-info { display:flex; align-items:center; justify-content:space-between; gap:12px; font:600 13px var(--sans); color:var(--ink2); }
  .style-toggle { display:inline-flex; align-items:center; gap:7px; border:1px solid var(--line2,var(--line)); border-radius:999px; padding:5px 10px; color:var(--ink); background:var(--win); }
  .style-toggle i { border:1.5px solid currentColor; width:9px; height:9px; border-radius:50%; }
  .style-toggle.active i { background:var(--active); border-color:var(--active); }
  .style-toggle:disabled { color:var(--ink2); cursor:default; }
  .ruta { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; padding-top: 14px; border-top: 1px solid var(--line); }
  /* Cada ruta abre el Explorador con el archivo seleccionado; la carpeta aparece al pasar el ratón. */
  .archivo {
    max-width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    margin-left: -8px;
    padding: 4px 8px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--ink2);
    font: 400 12px var(--mono);
    text-align: left;
    transition: background-color 0.15s, color 0.15s;
  }
  .archivo .txt { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .archivo :global(svg) { opacity: 0; transition: opacity 0.15s; }
  .archivo:hover { background: var(--hov); color: var(--ink); }
  .archivo:hover :global(svg), .archivo:focus-visible :global(svg) { opacity: 1; }
</style>
