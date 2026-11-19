<script>
  // Una fila de la lista: nombre pequeño encima y la fuente debajo.
  import { ChevronRight } from "@lucide/svelte";
  import { palabrasDe } from "../lib/organizacion.svelte.js";
  import Interruptor from "./Interruptor.svelte";
  import Corazon from "./Corazon.svelte";
  import { t } from "../lib/i18n.svelte.js";

  let { f, estilo, muestra, tamano, sel, fav, bloqueada, motivo, onsel, onetiqueta, onabrir, onalternar, onfav, onarrastre } = $props();

  const on = $derived(f.activa);
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="fila" class:sel onclick={onsel} ondblclick={onabrir} onpointerdown={onarrastre} data-familia={f.nombre} data-fila={f.claveVista || f.nombre}>
  <Interruptor parcial={!f.estiloVista && f.activa && f.estilos.some(s=>s.activa) && f.estilos.some(s=>!s.activa)} activa={f.activa} {bloqueada} titulo={bloqueada ? motivo : f.activa ? t("deactivate") : t("activate")} onclick={onalternar} />
  <div class="txt">
    <span class="nombre">{f.tituloVista ?? f.nombre} · {((f.estiloVista?.archivo||f.archivo)?.split(".").pop()||"").toUpperCase()} · {t("estilos.titulo",{n:f.estilos.length})}{#if f.error}<span class="roto"> · {t(f.error === "cloud-only" ? "cloudOnly" : "err.ilegible")}</span>{/if}</span>
    <span
      class="muestra"
      class:off={!on}
      style:font-family={estilo.familia}
      style:font-weight={estilo.peso}
      style:font-style={estilo.cursiva ? "italic" : "normal"}
      style:font-size="{tamano}px">{muestra}</span
    >
    <div class="tags">{#each palabrasDe(f).slice(0,8) as tag (tag)}<button title={tag} onclick={ev=>{ev.stopPropagation();onetiqueta(tag);}} ondblclick={ev=>ev.stopPropagation()}>{tag}</button>{/each}</div>
  </div>
  <button
    type="button"
    class="ico corazon"
    class:fav
    title={fav ? t("fav.quitar") : t("fav.poner")}
    aria-label={fav ? t("fav.quitar") : t("fav.poner")}
    onclick={(e) => (e.stopPropagation(), onfav())}
    ondblclick={(e) => e.stopPropagation()}
  >
    <Corazon lleno={fav} />
  </button>
  <button
    type="button"
    class="ico abrir"
    title={t("specimen")}
    aria-label={t("specimen")}
    onclick={(e) => (e.stopPropagation(), onabrir())}
    ondblclick={(e) => e.stopPropagation()}
  >
    <ChevronRight size={17} strokeWidth={2} />
  </button>
</div>

<style>
  .tags {display:flex;gap:4px;overflow:hidden;align-items:center;min-height:20px;} .tags button {flex:none;padding:2px 7px;border:0;border-radius:12px;background:var(--chip);color:var(--fuente-inactiva);font:500 11px var(--sans);}
  .fila {
    display: grid;
    grid-template-columns: 24px minmax(0, 1fr) 34px 34px;
    /* Tick, corazón y flecha centrados con toda la fila (nombre + muestra). */
    align-items: center;
    gap: 0 12px;
    padding: 8px 18px 8px 24px;
    transition: background-color 0.12s;
  }
  .fila:hover { background: var(--hov); }
  .fila.sel, .fila.sel:hover { background: var(--sel); }
  .txt { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .nombre { font-size: 10.5px; color: var(--fuente-inactiva); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .roto { color: var(--red); }
  /* Recorta solo a lo ancho (para los «…»): por arriba y por abajo los adornos de las caligráficas pueden salirse. */
  .muestra {
    line-height: 1.38;
    /* Aire encima para los adornos altos, sin pisar el nombre (la fila crece lo mismo en App.svelte). */
    padding-top: 0.2em;
    white-space: nowrap;
    overflow-x: clip;
    overflow-y: visible;
    text-overflow: ellipsis;
    color: var(--ink);
    transition: color 0.25s;
  }
  .muestra.off { color: var(--fuente-inactiva); }
  .ico {
    width: 32px;
    height: 32px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--ink3);
    opacity: 0.6;
    transition: background-color 0.15s, opacity 0.15s, color 0.15s;
  }
  .ico:hover { opacity: 1; background: var(--hov); }
  .abrir { opacity: 0.7; }
  .abrir:hover { color: var(--ink); }
  .corazon.fav { color: var(--red); opacity: 1; }
</style>
