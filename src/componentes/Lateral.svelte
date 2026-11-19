<script>
  import { SvelteSet } from "svelte/reactivity";
  import Marca from "./Marca.svelte";
  import { SlidersHorizontal, Sun, Moon, Plus, Files, FileType, Star, Clock, Layers, Library, Folder, FolderCog, Type } from "@lucide/svelte";
  import Segmentado from "./Segmentado.svelte";
  import Actualizacion from "./Actualizacion.svelte";
  import { ajustes, temaActual, ponerTema, version } from "../lib/ajustes.svelte.js";
  import { arrastre } from "../lib/arrastre.svelte.js";
  import { t } from "../lib/i18n.svelte.js";

  // seccion: instaladas | descubrir | favoritas | col:<id> | car:<ruta>
  let {
    seccion, cuentas, guardados, onguardado, colecciones, carpetas, pie, renombrando = $bindable(),
    onseccion, onnueva, onrenombrar, onanadirCarpeta, onajustes, onacerca,
  } = $props();

  const iconos = {colecciones:Layers,proveedores:Library,carpetas:Folder,inteligentes:FolderCog};

  const plegados = new SvelteSet();
  const excepciones = new SvelteSet();
  function alternar(conjunto, id) {
    if (conjunto.has(id)) conjunto.delete(id);
    else conjunto.add(id);
  }

  let borrador = $state("");
  let campo = $state();

  $effect(() => {
    if (renombrando && campo) {
      borrador = colecciones.find((c) => c.id === renombrando)?.nombre ?? "";
      campo.focus();
      campo.select();
    }
  });

  function terminar(guardar) {
    if (guardar) onrenombrar(renombrando, borrador);
    renombrando = null;
  }

  // Colección nueva: campo en línea; solo se crea con Enter y un nombre.
  let creando = $state(false);
  let nombreNuevo = $state("");
  let campoNuevo = $state();
  $effect(() => {
    if (creando && campoNuevo) campoNuevo.focus();
  });
  function empezarNueva() {
    nombreNuevo = "";
    creando = true;
  }
  function acabarNueva(crear) {
    const n = nombreNuevo.trim();
    creando = false;
    if (crear && n) onnueva(n);
  }
</script>

{#snippet item(texto, activo, ir, cuenta, extra = {})}
  <button
    type="button"
    class="item"
    class:activo
    class:diana={extra.diana}
    onclick={ir}
    ondblclick={extra.doble}
    title={extra.titulo}
    data-coleccion={extra.coleccion}
    aria-current={activo ? "page" : undefined}
  >
    {#if extra.icono}<extra.icono size={15} strokeWidth={1.8}/>{:else}<span class="punto" class:oculto={extra.oculto} class:verde={extra.verde} class:parcial={extra.parcial}></span>{/if}
    <span class="texto">{texto}</span>
    {#if cuenta != null && cuenta !== ""}<span class="cuenta num">{cuenta}</span>{/if}
  </button>
{/snippet}

{#snippet cabecera(id, texto)}
  {@const Icono=iconos[id]}
  <button class="titulo cabecera" type="button" aria-expanded={!plegados.has(id)} onclick={() => alternar(plegados, id)}>
    <span class="flecha" class:abierta={!plegados.has(id)}></span><Icono size={14} />{texto}
  </button>
{/snippet}

{#snippet rama(c, raiz, nivel)}
  {@const id = raiz + "|" + c.ruta}
  {@const abierta = nivel === 0 ? !excepciones.has(id) : excepciones.has(id)}
  {@const estado = c.activas === 0 ? t("sec.inactivas") : c.activas === c.totalEstilos ? t("nav.active") : t("nav.partial")}
  <div class="rama" style:padding-left={Math.min(nivel, 6) * 9 + "px"}>
    {#if c.hijos.length}
      <button class="desplegar" type="button" aria-label={c.ruta} aria-expanded={abierta} onclick={() => alternar(excepciones, id)}>
        <span class="flecha" class:abierta></span>
      </button>
    {:else}
      <span class="hueco"></span>
    {/if}
    {@render item(c.nombre, seccion === "car:" + c.ruta, () => onseccion("car:" + c.ruta), c.cuenta,
      { titulo: c.ruta + " · " + estado, verde: c.activas > 0 && c.activas === c.totalEstilos, parcial: c.activas > 0 && c.activas < c.totalEstilos })}
  </div>
  {#if abierta}
    {#each c.hijos as hijo (hijo.ruta)}{@render rama(hijo, raiz, nivel + 1)}{/each}
  {/if}
{/snippet}

<aside>
  <div class="cabeza" data-tauri-drag-region>
    <Marca ancho={116} />
  </div>

  <nav>
    <div class="grupo">
      {@render item(t("nav.all"), seccion === "instaladas", () => onseccion("instaladas"), cuentas.instaladas, { icono:Files })}
      {@render item(t("sec.sistema"),seccion==="sistema",()=>onseccion("sistema"),cuentas.sistema,{icono:FileType})}
      {@render item(t("sec.recientes"), seccion === "recientes", () => onseccion("recientes"), cuentas.recientes, { icono:Clock })}
      {@render item(t("nav.favorites"), seccion === "favoritas", () => onseccion("favoritas"), cuentas.favoritas,{icono:Star})}
      {@render item(t("nav.active"), seccion === "activas", () => onseccion("activas"), cuentas.activas, { verde: true })}
      {@render item(t("sec.inactivas"), seccion === "inactivas", () => onseccion("inactivas"), cuentas.inactivas)}
    </div>

    <div class="grupo">
      {@render cabecera("colecciones", t("grupo.colecciones"))}
      {#if !plegados.has("colecciones")}
      {#each colecciones as c (c.id)}
        {#if renombrando === c.id}
          <input
            class="renombrar"
            bind:this={campo}
            bind:value={borrador}
            onkeydown={(e) => {
              if (e.key === "Enter") terminar(true);
              if (e.key === "Escape") (e.stopPropagation(), terminar(false));
            }}
            onblur={() => terminar(true)}
          />
        {:else}
          {@render item(c.nombre, seccion === "col:" + c.id, () => onseccion("col:" + c.id), c.fuentes.length, {
            doble: () => (renombrando = c.id),
            coleccion: c.id,
            diana: arrastre.activo && arrastre.sobre === c.id,
            titulo: t("col.pista"),
          })}
        {/if}
      {/each}
      {#if creando}
        <input
          class="renombrar"
          bind:this={campoNuevo}
          bind:value={nombreNuevo}
          placeholder={t("col.nuevaNombre")}
          aria-label={t("col.nueva")}
          onkeydown={(e) => {
            if (e.key === "Enter") acabarNueva(true);
            if (e.key === "Escape") (e.stopPropagation(), acabarNueva(false));
          }}
          onblur={() => acabarNueva(false)}
        />
      {/if}
      {#if !creando}
        <button class="nueva" type="button" onclick={empezarNueva}>{t("col.nueva")}</button>
      {/if}
      {/if}
    </div>

    <div class="grupo">
      {@render cabecera("proveedores", t("nav.providers"))}
      {#if !plegados.has("proveedores")}
        {@render item("Google Fonts", seccion === "descubrir", () => onseccion("descubrir"), cuentas.descubrir,{icono:Type})}
      {/if}
    </div>

    <div class="grupo">
      {@render cabecera("inteligentes",t("filter.smart"))}
      {#if !plegados.has("inteligentes")}
        {#each guardados as g (g.nombre)}
          <div class="saved">{@render item(g.nombre,false,()=>onguardado(g),null,{icono:SlidersHorizontal})}<button class="quitar" aria-label={t("filter.removeSaved")+" · "+g.nombre} onclick={()=>ajustes.filtrosGuardados=guardados.filter(v=>v.nombre!==g.nombre)}>×</button></div>
        {/each}
        {#if !guardados.length}<small class="nota">{t("filter.smartHint")}</small>{/if}
      {/if}
    </div>
    <div class="grupo">
      {@render cabecera("carpetas", t("nav.folders"))}
      {#if !plegados.has("carpetas")}
        {#each carpetas as c (c.ruta)}{@render rama(c, c.ruta, 0)}{/each}
      {/if}
    </div>
  </nav>

  <div class="pie">
    <button type="button" class="anadir" onclick={() => onanadirCarpeta()}><Plus size={14} />{t("nav.add")}</button>
    <Actualizacion />
    <span class="nota num">{pie}</span>
    <div class="fila">
      <button type="button" class="ajustes" onclick={onajustes}>
        <SlidersHorizontal size={15} strokeWidth={1.9} />{t("settings")}
      </button>
      <Segmentado
        etiqueta={t("aj.tema")}
        opciones={[
          { valor: "light", icono: Sun, titulo: t("light") },
          { valor: "dark", icono: Moon, titulo: t("dark") },
        ]}
        valor={temaActual()}
        onelegir={ponerTema}
        ancho={30}
        alto={22}
      />
    </div>
    <button type="button" class="version" onclick={onacerca}>v{version} · {t("about")}</button>
  </div>
</aside>

<style>
  aside {
    background: var(--side);
    border-right: 1px solid var(--line);
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 0 10px 14px;
    transition: background-color 0.3s;
  }
  .cabeza { flex: none; height: 52px; display: flex; align-items: center; gap: 8px; padding: 0 8px; }
  nav { flex: 1 1 auto; min-height: 0; overflow-y: auto; margin: 0 -6px; padding: 0 6px 6px; }
  .saved {display:flex;align-items:center;} .saved :global(.item) {flex:1;min-width:0;} .quitar {border:0;color:var(--ink2);background:none;}
  .grupo { display: flex; flex-direction: column; gap: 1px; margin-top: 14px; }
  .grupo:first-child { margin-top: 2px; }
  .titulo { padding: 0 10px 6px; font-size: 10.5px; font-weight: 600; letter-spacing: 0.08em; text-transform: uppercase; color: var(--ink3); }
  .item {
    display: flex;
    align-items: center;
    gap: 9px;
    height: 26px;
    padding: 0 10px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--ink2);
    font: 500 13px var(--sans);
    text-align: left;
    transition: background-color 0.15s, color 0.15s, box-shadow 0.15s;
  }
  .item:hover { background: var(--hov); }
  .item.activo { background: var(--chip); color: var(--ink); font-weight: 600; }
  .item.diana { box-shadow: inset 0 0 0 1.5px var(--red); }
  .punto { width: 10px; height: 10px; flex: none; border-radius: 50%; box-shadow: inset 0 0 0 1.5px var(--fuente-inactiva); }
  .punto.oculto { visibility: hidden; }
  .punto.verde { background: var(--active); box-shadow: none; }
  .punto.parcial { box-shadow: inset 0 0 0 1.5px var(--active); }
  .cabecera, .desplegar, .anadir, .nueva { background: transparent; border: 0; color: var(--ink2); cursor: pointer; }
  .cabecera { display: flex; align-items: center; gap: 8px; min-height: 23px; text-align: left; }
  .flecha { width: 0; height: 0; border-top: 4px solid transparent; border-bottom: 4px solid transparent; border-left: 5px solid currentColor; }
  .flecha.abierta { transform: rotate(90deg); }
  .rama { display: flex; align-items: center; min-width: 0; }
  .rama .item { flex: 1; min-width: 0; padding-left: 4px; }
  .desplegar, .hueco { flex: 0 0 15px; }
  .desplegar { display: grid; place-items: center; height: 24px; padding: 0; }
  .anadir { display: flex; align-items: center; justify-content: center; gap: 5px; padding: 5px; font-weight: 600; }
  .nueva { text-align: left; padding: 5px 12px; color: var(--ink3); }
  .anadir:hover, .nueva:hover, .desplegar:hover, .cabecera:hover { background: var(--hov); }
  .texto { flex: 1 1 auto; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; padding-right: 1px; }
  .cuenta { margin-left: auto; font-size: 11.5px; color: var(--ink3); }
  .renombrar {
    height: 30px;
    padding: 0 10px;
    border: 0;
    border-radius: 6px;
    background: var(--field);
    box-shadow: 0 0 0 1.5px var(--red);
    font: 600 13px var(--sans);
    color: var(--ink);
    outline: 0;
    width: 100%;
  }
  .renombrar::placeholder { color: var(--ink3); font-weight: 500; }
  .pie { flex: none; display: flex; flex-direction: column; gap: 10px; padding: 10px 4px 0; }
  .nota { font-size: 11.5px; color: var(--ink3); padding: 0 6px; }
  .fila { display: flex; align-items: center; gap: 6px; }
  .ajustes {
    flex: 1;
    height: 32px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 10px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--ink2);
    font: 600 12.5px var(--sans);
    transition: background-color 0.15s, color 0.15s;
  }
  .ajustes:hover { background: var(--hov); color: var(--ink); }
  .version { align-self: flex-start; padding: 0 6px; border: 0; background: transparent; color: var(--ink3); font: 500 11px var(--mono); transition: color 0.15s; }
  .version:hover { color: var(--ink); }
</style>
