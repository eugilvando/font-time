<script>
  import { iniciarAdobe } from "./lib/adobe.svelte.js";
  import { iniciarSync } from "./lib/sync.svelte.js";
  import Marca from "./componentes/Marca.svelte";
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { Search, List, LayoutGrid, FolderOpen, Trash2, X, Heart, WifiOff, Layers, Type, LoaderCircle } from "@lucide/svelte";

  import Filtros from "./componentes/Filtros.svelte";
  import { vacios,cantidad,coincideEstilo } from "./lib/filtros.js";
  import Lateral from "./componentes/Lateral.svelte";
  import Controles from "./componentes/Controles.svelte";
  import Avisos from "./componentes/Avisos.svelte";
  import Segmentado from "./componentes/Segmentado.svelte";
  import ListaVirtual from "./componentes/ListaVirtual.svelte";
  import Fila from "./componentes/Fila.svelte";
  import Tarjeta from "./componentes/Tarjeta.svelte";
  import { iniciarActualizador } from "./lib/actualizador.svelte.js";
  import TarjetaGoogle from "./componentes/TarjetaGoogle.svelte";
  import FilaGoogle from "./componentes/FilaGoogle.svelte";
  import Vacio from "./componentes/Vacio.svelte";
  import Esqueleto from "./componentes/Esqueleto.svelte";
  import MenuFuentes from "./componentes/MenuFuentes.svelte";
  import BarraSeleccion from "./componentes/BarraSeleccion.svelte";
  import Ajustes from "./vistas/Ajustes.svelte";
  import Especimen from "./vistas/Especimen.svelte";
  import Comparar from "./vistas/Comparar.svelte";

  import { ajustes, esWindows, enTauri, temaActual } from "./lib/ajustes.svelte.js";
  import { arbolCarpetas, dentro, enCarpeta } from "./lib/carpetas.js";
  import { t } from "./lib/i18n.svelte.js";
  import { avisar } from "./lib/avisos.svelte.js";
  import { fuentes, iniciarFuentes, actualizarFuentes, alternar, agregarArchivos, estiloDe, ponerLista, llamar } from "./lib/fuentes.svelte.js";
  import { biblioteca, iniciarBiblioteca, alternarFavorita, aFavoritas, nuevaColeccion, renombrar, eliminarColeccion, aColeccion } from "./lib/biblioteca.svelte.js";
  import { descubrir, cargarCatalogo, pedirVistas, precargar, instalarGoogle } from "./lib/descubrir.svelte.js";
  import { arrastre, empezarArrastre } from "./lib/arrastre.svelte.js";
  import { comboDe } from "./lib/atajos.js";

  // ---- estado de la vista ----
  // instaladas | descubrir | favoritas | col:<id> | car:<ruta>
  let seccion = $state(ajustes.alAbrir === "carpeta" && ajustes.carpetaPrincipal ? "car:" + ajustes.carpetaPrincipal : ajustes.alAbrir === "favoritas" ? "favoritas" : "instaladas");
  let filtros = $state(vacios());
  let q = $state("");
  let muestraPropia = $state("");

  let orden = $state("popularidad");
  const sel = new SvelteSet();
  let detalle = $state(null);
  let estiloInicial = $state(null);
  let comparar = $state([]);
  let ajustesEn = $state(null);
  let colAbierta = $state(false);
  let renombrando = $state(null);
  let borrando = $state(null);
  let soltando = $state(false);
  let carpetas = $state([]);
  let leyendo = $state(null);
  let buscador = $state();

  const sinMovimiento = matchMedia("(prefers-reduced-motion: reduce)").matches;
  let splash = $state(!sinMovimiento);
  if (!sinMovimiento) setTimeout(() => (splash = false), 800);

  import { muestrasActuales, filasDeFamilias, palabrasDe, grupoDe } from "./lib/organizacion.svelte.js";
  const muestras = $derived(muestrasActuales());
  const muestraDefecto = $derived(muestras[0] || t("defaultSample"));
  const muestra = $derived(muestraPropia || muestraDefecto);
  const tamano = $derived(ajustes.tamano);

  const enDescubrir = $derived(seccion === "descubrir");
  const coleccion = $derived(seccion.startsWith("col:") ? biblioteca.colecciones.find((c) => "col:" + c.id === seccion) : null);
  const carpeta = $derived(seccion.startsWith("car:") ? seccion.slice(4) : null);
  const detalleF = $derived(detalle ? fuentes.lista.find((f) => f.nombre === detalle) : null);
  const comparadas = $derived(comparar.map((n) => fuentes.lista.find((f) => f.nombre === n)).filter(Boolean));

  const bloqueada = (f) => f.estiloVista ? !f.estiloVista.editable : f.externa || f.error === "cloud-only" || f.origen === "sis" && (!ajustes.desbloqueadas || f.imprescindible);
  const motivo = (f) => f.estiloVista && !f.estiloVista.editable ? t("org.styleLock") : f.error === "cloud-only" ? t("cloudOnly") : f.externa ? t("externalFont") : (f.imprescindible ? t("candado.imprescindible") : t("candado.sistema"));
  // Las del sistema con candado no se enseñan salvo que se pida en ajustes
  // (en favoritas y colecciones sí: ahí las has puesto tú).
  const visible = (f) => ajustes.verBloqueadas || f.origen !== "sis" || !bloqueada(f);
  // Por archivo, no por origen: si la familia también está instalada, cuenta como instalada.
  const favs = $derived(new Set(biblioteca.favoritas));

  // ---- listas ----
  const base = $derived.by(() => {
    const l = fuentes.lista;
    if (seccion === "activas") return ajustes.agrupar ? instaladas.filter((f) => f.activa) : instaladas;
    if (seccion === "inactivas") return ajustes.agrupar ? instaladas.filter((f) => !f.activa) : instaladas;
    if (seccion === "sistema") return l.filter(f=>f.origen==="sis");
    if (seccion === "recientes") return recientes;
    if (seccion === "favoritas") return l.filter((f) => favs.has(f.nombre));
    if (coleccion) return l.filter((f) => coleccion.fuentes.includes(f.nombre));
    if (carpeta) return l.filter((f) => enCarpeta(f, carpeta));
    return instaladas;
  });
  const ql = $derived(q.trim().toLowerCase());
  function enOrigen(s) {
    return !carpeta || (s.fuentes?.length?s.fuentes:[s.archivo]).some(r=>r && dentro(r,carpeta));
  }
  const estilosBase=$derived(base.flatMap(f=>f.estilos.filter(enOrigen)));
  const lista=$derived(base.filter(f=>{
    if(filtros.favoritas && !favs.has(f.nombre) || filtros.categorias.length && !filtros.categorias.includes(f.categoria)) return false;
    if(ql || filtros.etiqueta) {
      const tags=palabrasDe(f);
      if(filtros.etiqueta && !tags.some(s=>s.toLowerCase()===filtros.etiqueta.toLowerCase())) return false;
      if(ql && ![f.nombre,grupoDe(f.nombre),...tags].some(s=>s.toLowerCase().includes(ql))) return false;
    }
    if(!f.estilos.length) return coincideEstilo(f,filtros);
    return f.estilos.some(s=>enOrigen(s) && coincideEstilo(s,filtros));
  }));
  const instaladasNombres = $derived(new Set(fuentes.lista.map((f) => f.nombre.toLowerCase())));
  // Favoritas de Google Fonts sin instalar (marcarlas no instala nada: es una lista de deseos).
  const favGoogle = $derived(
    seccion === "favoritas"
      ? descubrir.lista.filter(
          (g) =>
            favs.has(g.nombre) &&
            !instaladasNombres.has(g.nombre.toLowerCase()) &&
            (!filtros.categorias.length || filtros.categorias.includes(g.categoria)) &&
            (!ql || g.nombre.toLowerCase().includes(ql)) &&
            filtros.estado !== "activas" && !filtros.variable && !filtros.color && !filtros.duplicada && !filtros.etiqueta && !Object.keys(filtros.rangos).length,
        )
      : [],
  );
  // Lo que se pinta: en favoritas, las instaladas y las de Google por orden alfabético.
  const familiasVista = $derived(
    favGoogle.length
      ? [...lista, ...favGoogle.map((g) => ({ google: g, nombre: g.nombre }))].sort((a, b) => a.nombre.localeCompare(b.nombre))
      : lista,
  );
  const gruposAbiertos = new SvelteSet();
  const listaVista=$derived(filasDeFamilias(familiasVista,gruposAbiertos).filter(f=>
    !f.estiloVista || enOrigen(f.estiloVista) && coincideEstilo(f.estiloVista,filtros)));
  const claveFila = f => f.claveVista || f.nombre;
  const seleccionadas = $derived(listaVista.filter(f => !f.google && !f.grupo && sel.has(claveFila(f))));
  const familiasSeleccionadas = $derived([...new Set(seleccionadas.map(f => f.nombre))]);
  const idEstilo = f => ({familia:f.nombre, nombre:f.estiloVista.nombre, peso:f.estiloVista.peso, cursiva:f.estiloVista.cursiva});
  // El catálogo solo hace falta si hay favoritas que no están instaladas.
  $effect(() => {
    if (seccion === "favoritas" && biblioteca.favoritas.some((n) => !instaladasNombres.has(n.toLowerCase()))) cargarCatalogo();
  });
  const listaGoogle = $derived.by(() => {
    const l = descubrir.lista.filter((g) => (!filtros.categorias.length || filtros.categorias.includes(g.categoria)) && (!ql || g.nombre.toLowerCase().includes(ql)));
    if (orden === "popularidad") return [...l].sort((a, b) => a.popularidad - b.popularidad);
    if (orden === "recientes") return [...l].sort((a, b) => (b.fecha > a.fecha ? 1 : b.fecha < a.fecha ? -1 : 0));
    return [...l].sort((a, b) => a.nombre.localeCompare(b.nombre));
  });

  const instaladas = $derived(fuentes.lista.filter((f) => visible(f) || carpetas.some((c) => enCarpeta(f, c.ruta))));
  const activasN = $derived(instaladas.filter((f) => f.activa).length);
  const recientes = $derived.by(() => {
    const porNombre = new Map(instaladas.map((f) => [f.nombre, f]));
    return biblioteca.recientes.map((n) => porNombre.get(n)).filter(Boolean);
  });
  const estadosInventario = $derived(ajustes.agrupar ? null : instaladas.flatMap(f => f.estilos.length ? f.estilos : [f]));
  const totalVista = $derived(estadosInventario?.length ?? instaladas.length);
  const activasVista = $derived(estadosInventario?.filter(s => s.activa).length ?? activasN);
  const cuentas = $derived({ sistema:fuentes.lista.filter(f=>f.origen==="sis").length, instaladas: totalVista, activas: activasVista, recientes: recientes.length,
    inactivas: totalVista - activasVista, descubrir: descubrir.lista.length || "", favoritas: biblioteca.favoritas.length });
  const carpetasCuenta = $derived(arbolCarpetas(carpetas, fuentes.directorios, fuentes.lista));

  const cabecera = $derived(
    comparadas.length
      ? t("comparar")
      : detalleF
        ? t("especimen")
        : enDescubrir
            ? t("google")
            : seccion === "favoritas"
              ? t("sec.favoritas")
              : coleccion
                ? coleccion.nombre
                : carpeta
                  ? carpeta
                  : t("sec." + seccion),
  );
  const vistaNormal = $derived(!detalleF && !comparadas.length);

  // Vistas previas de Descubrir: solo lo que se ve.
  let vistasGoogle = [];
  function alVerGoogle(items) {
    vistasGoogle = items;
    pedirVistas(items.map((g) => g.nombre), muestra);
  }
  function alVerFav(items) {
    vistasGoogle = items.filter((it) => it.google).map((it) => it.google);
    if (vistasGoogle.length) pedirVistas(vistasGoogle.map((g) => g.nombre), muestra);
  }
  $effect(() => {
    // Si cambia el texto de prueba, se pide solo lo que falte de lo que se ve.
    const m = muestra;
    if (enDescubrir || seccion === "favoritas") pedirVistas(vistasGoogle.map((g) => g.nombre), m);
  });

  // ---- acciones ----
  function irA(s) {
    seccion = s;
    q = "";
    filtros = vacios();
    if (s==="activas" || s==="inactivas") filtros.estado=s;
    detalle = null;
    comparar = [];
    sel.clear();
    colAbierta = false;
    if (s === "descubrir") cargarCatalogo();
  }

  function aplicarGuardado(g) {
    irA("instaladas");
    filtros=structuredClone($state.snapshot(g.filtros));
    q=g.busqueda||"";
  }

  function abrirFamilia(f) {
    estiloInicial = f.estiloVista ?? null;
    detalle = f.nombre;
    sel.clear();
    biblioteca.recientes = [f.nombre, ...biblioteca.recientes.filter((n) => n !== f.nombre)].slice(0, 100);
  }

  async function alternarUna(f) {
    const activar = !f.activa;
    const r = await alternar(f.estiloVista ? [] : [f.nombre], activar, ajustes.desbloqueadas, f.estiloVista ? [idEstilo(f)] : []);
    if (r?.cambiadas) avisar("ok", t(activar ? "ok.activada" : "ok.desactivada", { n: f.nombre }), activar ? t("ok.yaSale") : "");
  }

  async function alternarVarias(fs, activar, limpiar = true) {
    const tocar = fs.filter(f => !bloqueada(f) && f.activa !== activar);
    if (limpiar) sel.clear();
    colAbierta = false;
    if (!tocar.length) return;
    const familias = tocar.filter(f => !f.estiloVista).map(f => f.nombre);
    const estilos = tocar.filter(f => f.estiloVista).map(idEstilo);
    const r = await alternar(familias, activar, ajustes.desbloqueadas, estilos);
    if (r?.cambiadas) avisar("ok", t(activar ? "ok.activadas" : "ok.desactivadas", { n:r.cambiadas }), "");
  }

  function aCol(id, nombres) {
    let c = biblioteca.colecciones.find((x) => x.id === id);
    if (!c) {
      id = nuevaColeccion(t("col.nombre", { n: biblioteca.colecciones.length + 1 }));
      c = biblioteca.colecciones.find((x) => x.id === id);
    }
    aColeccion(id, nombres);
    avisar("ok", t("ok.aColeccion", { n: nombres.length === 1 ? nombres[0] : t("fuentesN", { n: nombres.length }), c: c.nombre }), "");
  }

  function crearColeccion(nombre) {
    const id = nuevaColeccion(nombre);
    irA("col:" + id);
  }

  let temporizadorBorrar;
  function borrarColeccion() {
    if (!coleccion) return;
    if (borrando !== coleccion.id) {
      borrando = coleccion.id;
      clearTimeout(temporizadorBorrar);
      temporizadorBorrar = setTimeout(() => (borrando = null), 3000);
      return;
    }
    eliminarColeccion(coleccion.id);
    borrando = null;
    irA("instaladas");
    avisar("ok", t("ok.colEliminada"), t("ok.siguenInstaladas"));
  }

  async function activarColeccion(activar) {
    if (!coleccion) return;
    const n = coleccion.fuentes;
    const fs = n.map((x) => fuentes.lista.find((f) => f.nombre === x)).filter((f) => f && !bloqueada(f) && f.activa !== activar);
    if (fs.length) await alternar(fs.map((f) => f.nombre), activar, ajustes.desbloqueadas);
    if (activar) avisar("ok", t("ok.colActivada"), t("ok.colListas", { n: n.length }));
    else avisar("ok", t("ok.colDesactivada"), "");
  }

  async function anadirCarpeta(principal = false) {
    if (!enTauri) return;
    const seleccion = await llamar("anadir_carpeta");
    if (!seleccion) return;
    carpetas = seleccion.carpetas;
    if (principal === true || !ajustes.carpetaPrincipal) {
      ajustes.carpetaPrincipal = seleccion.ruta;
      ajustes.alAbrir = "carpeta";
    }
    ajustesEn = null;
    irA("car:" + seleccion.ruta);
    leyendo = seleccion.ruta;
    try {
      await actualizarFuentes();
    } finally {
      leyendo = null;
    }
    avisar("ok", t("ok.carpeta"), t("ok.carpetaTexto"));
  }

  async function quitarCarpeta(ruta) {
    carpetas = await llamar("quitar_carpeta", { ruta });
    if (ajustes.carpetaPrincipal === ruta) {
      ajustes.carpetaPrincipal = "";
      if (ajustes.alAbrir === "carpeta") ajustes.alAbrir = "instaladas";
    }
    await actualizarFuentes();
    if (carpeta && (carpeta === ruta || dentro(carpeta, ruta)) && !carpetas.some((c) => carpeta === c.ruta || dentro(carpeta, c.ruta))) irA("instaladas");
    avisar("ok", t("ok.carpetaQuitada"), t("ok.carpetaQuitadaTexto"));
  }

   function seleccionar(f) {
    if (sel.has(claveFila(f))) sel.delete(claveFila(f));
    else sel.add(claveFila(f));
    colAbierta = false;
  }

  function arrastrar(e, f) {
    empezarArrastre(
      e,
      () => (sel.has(claveFila(f)) ? familiasSeleccionadas : [f.nombre]),
      (id, nombres) => {
        aCol(id, nombres);
        sel.clear();
      },
    );
  }

  // ---- Ctrl + rueda: tamaño de las muestras, no zoom de la ventana ----
  // Con addEventListener porque hace falta passive: false para poder cancelar el zoom.
  $effect(() => {
    const rueda = (e) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      const paso = e.deltaY < 0 ? 2 : -2;
      ajustes.tamano = Math.min(96, Math.max(10, Math.round(ajustes.tamano) + paso));
    };
    window.addEventListener("wheel", rueda, { passive: false });
    return () => window.removeEventListener("wheel", rueda);
  });

  // ---- teclado ----
  function tecla(e) {
    if (document.querySelector("dialog[open], [popover]:popover-open")) return;
    if (ajustesEn || splash) return;
    const combo = comboDe(e) ?? "";
    const tag = e.target?.tagName ?? "";
    const escribiendo = tag === "INPUT" || tag === "TEXTAREA";
    if (combo === ajustes.atajos.buscar) {
      e.preventDefault();
      buscador?.focus();
      buscador?.select();
      return;
    }
    if (e.key === "Escape") {
      if (escribiendo) return e.target.blur();
      if (colAbierta) colAbierta = false;
      else if (comparar.length) comparar = [];
      else if (detalle) detalle = null;
      else sel.clear();
      return;
    }
    if (escribiendo || !vistaNormal || enDescubrir) return;
    if (combo === ajustes.atajos.todas) {
      e.preventDefault();
      for (const f of listaVista) if (!f.google && !f.grupo) sel.add(claveFila(f));
      return;
    }
    if (combo === ajustes.atajos.alternar && sel.size) {
      e.preventDefault();
      const algunaApagada = seleccionadas.some(f => !bloqueada(f) && !f.activa);
      alternarVarias(seleccionadas, algunaApagada, false);
    }
  }

  // ---- bandeja ----
  // El menú de la bandeja enseña cómo están las fuentes, con el tema y el idioma de la app.
  $effect(() => {
    if (!enTauri || !esWindows) return;
    const n = carpetas.length;
    const estado = n === 1 ? t("menu.carpeta") : n ? t("menu.carpetas", { n }) : t("menu.activas", { a: activasN, n: instaladas.length });
    const datos = { estado, carpetas: n, tema: temaActual(), idioma: ajustes.idioma };
    const id = setTimeout(() => llamar("poner_bandeja", datos).catch(() => {}), 300);
    return () => clearTimeout(id);
  });

  // ---- arranque ----
  onMount(() => {
    let dejarSync;
    let montada = true;
    iniciarBiblioteca().then(() => { if (montada) dejarSync = iniciarSync(); });
    const dejarAdobe = iniciarAdobe();
    iniciarActualizador();
    iniciarFuentes();
    precargar(muestra);
    if (!enTauri) return;
    llamar("carpetas").then((l) => (carpetas = l));
    // Lo que se pide desde el menú de la bandeja (la ventana ya está a la vista).
    let quitarBandeja;
    import("@tauri-apps/api/event").then(async ({ listen }) => {
      quitarBandeja = await listen("bandeja", ({ payload }) => {
        if (payload === "buscar") {
          ajustesEn = null;
          if (seccion !== "instaladas") irA("instaladas");
          setTimeout(() => buscador?.focus(), 50);
        } else if (payload === "descubrir") {
          ajustesEn = null;
          irA("descubrir");
        } else if (payload === "ajustes") ajustesEn = "ajustes";
      });
    });
    let quitar;
    import("@tauri-apps/api/webview").then(async ({ getCurrentWebview }) => {
      // Archivos arrastrados desde fuera (el explorador). Lo de dentro de la
      // app no usa el arrastre del navegador, así que aquí nunca llega.
      quitar = await getCurrentWebview().onDragDropEvent(async ({ payload: p }) => {
        if (p.type === "enter") soltando = (p.paths ?? []).length > 0;
        else if (p.type === "leave") soltando = false;
        else if (p.type === "drop") {
          soltando = false;
          const rutas = (p.paths ?? []).filter((r) => /\.(ttf|otf|ttc|otc)$/i.test(r));
          if (!rutas.length) return avisar("err", t("err.noFuente"), t("err.noFuenteTexto"));
          const r = await agregarArchivos(rutas);
          if (!r || !r.instaladas.length) return;
          const titulo = r.instaladas.length === 1 ? t("ok.instalada", { n: r.instaladas[0] }) : t("ok.instaladas", { n: r.instaladas.length });
          avisar("ok", titulo, t("catalogHint"));
        }
      });
    });
    return () => {
      montada = false;
      dejarSync?.();
      dejarAdobe();
      quitar?.();
      quitarBandeja?.();
    };
  });
</script>

<svelte:window onkeydown={tecla} />
<MenuFuentes lista={listaVista} seleccion={seleccionadas} {bloqueada} onalternar={alternarVarias} onabrir={abrirFamilia}/>

{#snippet fila(f)}
  <Fila onetiqueta={s=>filtros.etiqueta=s}
    {f}
    estilo={estiloDe(f, f.estiloVista)}
    {muestra}
    {tamano}
    sel={sel.has(claveFila(f))}
    fav={favs.has(f.nombre)}
    bloqueada={bloqueada(f)}
    motivo={motivo(f)}
    onsel={() => seleccionar(f)}
    onabrir={() => abrirFamilia(f)}
    onalternar={() => alternarUna(f)}
    onfav={() => alternarFavorita(f.nombre)}
    onarrastre={(e) => arrastrar(e, f)}
  />
{/snippet}

{#snippet tarjeta(f)}
  <Tarjeta
    {f}
    estilo={estiloDe(f, f.estiloVista)}
    sel={sel.has(claveFila(f))}
    fav={favs.has(f.nombre)}
    bloqueada={bloqueada(f)}
    motivo={motivo(f)}
    onsel={() => seleccionar(f)}
    onabrir={() => abrirFamilia(f)}
    onalternar={() => alternarUna(f)}
    onfav={() => alternarFavorita(f.nombre)}
    onarrastre={(e) => arrastrar(e, f)}
  />
{/snippet}

{#snippet filaGoogle(g)}
  <FilaGoogle
    {g}
    {muestra}
    {tamano}
    instalada={instaladasNombres.has(g.nombre.toLowerCase())}
    pct={descubrir.descargas[g.nombre]}
    fav={favs.has(g.nombre)}
    onfav={() => alternarFavorita(g.nombre)}
    oninstalar={() => instalarGoogle(g.nombre, ponerLista)}
  />
{/snippet}

{#snippet grupoPersonal(it)}
  <button class="grupo-personal" aria-expanded={gruposAbiertos.has(it.grupo)} onclick={() => gruposAbiertos.has(it.grupo) ? gruposAbiertos.delete(it.grupo) : gruposAbiertos.add(it.grupo)}>
    <span>{gruposAbiertos.has(it.grupo) ? "▾" : "▸"} {it.grupo}</span>
    <small>{t("org.groupMembers")} · {it.cantidad}</small>
  </button>
{/snippet}

{#snippet filaFav(it)}
  {#if it.grupo}{@render grupoPersonal(it)}
  {:else if it.google}
    <FilaGoogle
      g={it.google}
      {muestra}
      {tamano}
      sinInstalar
      instalada={false}
      pct={descubrir.descargas[it.nombre]}
      fav={favs.has(it.nombre)}
      onfav={() => alternarFavorita(it.nombre)}
      oninstalar={() => instalarGoogle(it.nombre, ponerLista)}
    />
  {:else}
    {@render fila(it)}
  {/if}
{/snippet}

{#snippet tarjetaFav(it)}
  {#if it.grupo}{@render grupoPersonal(it)}
  {:else if it.google}
    <TarjetaGoogle
      g={it.google}
      pct={descubrir.descargas[it.nombre]}
      fav={favs.has(it.nombre)}
      onfav={() => alternarFavorita(it.nombre)}
      oninstalar={() => instalarGoogle(it.nombre, ponerLista)}
    />
  {:else}
    {@render tarjeta(it)}
  {/if}
{/snippet}

<div class="ventana">
  <Lateral
    {seccion}
    {cuentas}
    guardados={ajustes.filtrosGuardados} onguardado={aplicarGuardado}
    colecciones={biblioteca.colecciones}
    carpetas={carpetasCuenta}
    pie={t("activas", { n: activasVista, total: totalVista })}
    bind:renombrando
    onseccion={irA}
    onnueva={crearColeccion}
    onrenombrar={renombrar}
    onanadirCarpeta={anadirCarpeta}
    onajustes={() => (ajustesEn = "ajustes")}
    onacerca={() => (ajustesEn = "acerca")}
  />

  <main>
    <header data-tauri-drag-region>
      {#if vistaNormal}
        <label class="campo buscar">
          <Search size={13} strokeWidth={2} />
          <input bind:this={buscador} bind:value={q} placeholder={enDescubrir ? t("searchGoogle") : t("search")} spellcheck="false" />
        </label>
        {#if !enDescubrir}<Filtros bind:filtros bind:busqueda={q} estilos={estilosBase} estadoFijo={seccion==="activas"||seccion==="inactivas"}/>{/if}
        <label class="campo prueba">
          <span class="a">A</span>
          <input list="muestras-guardadas" bind:value={muestraPropia} placeholder={t("typeSomething")} spellcheck="false" />
          {#if muestraPropia}
            <button type="button" class="reset" title={t("resetText")} aria-label={t("resetText")} onclick={() => (muestraPropia = "")}><X size={9} strokeWidth={3} /></button>
          {/if}
        </label>
        <datalist id="muestras-guardadas">{#each muestras as frase}<option value={frase}></option>{/each}</datalist>
        <div class="tamano" title={t("tamano")}>
          <span class="a pq">A</span>
          <input class="rango" type="range" min="10" max="96" bind:value={ajustes.tamano} aria-label={t("tamano")} />
          <span class="a gr">A</span>
          <span class="num n">{tamano}</span>
        </div>
      {/if}
      <div class="hueco" data-tauri-drag-region></div>
      {#if vistaNormal && !enDescubrir}
        <div class="vista">
          <Segmentado
            etiqueta={t("list")}
            opciones={[
              { valor: "lista", icono: List, titulo: t("list") },
              { valor: "cuadricula", icono: LayoutGrid, titulo: t("grid") },
            ]}
            valor={ajustes.vista}
            onelegir={(v) => (ajustes.vista = v)}
            ancho={34}
            alto={24}
          />
        </div>
      {/if}
      {#if esWindows}<Controles />{/if}
    </header>

    <div class="titulo">
      <span class="etiqueta" class:ruta-cabecera={carpeta && vistaNormal} title={cabecera}>{cabecera}</span>
      {#if vistaNormal}<span class="cuenta num">{enDescubrir ? listaGoogle.length : listaVista.length}</span>{/if}
      <div class="derecha">
        {#if vistaNormal && enDescubrir}
          <Segmentado
            etiqueta={t("orden.popularidad")}
            opciones={["popularidad", "recientes", "alfabetico"].map((v) => ({ valor: v, etiqueta: t("orden." + v) }))}
            valor={orden}
            onelegir={(v) => (orden = v)}
            ancho={84}
            alto={28}
            fuente="600 12px"
          />
        {/if}
        {#if coleccion && vistaNormal}
          <span class="sep"></span>
          <button type="button" class="pild tinta" onclick={() => activarColeccion(true)}>{t("activateAll")}</button>
          <button type="button" class="pild" onclick={() => activarColeccion(false)}>{t("deactivate")}</button>
          <button type="button" class="pild borrar" class:seguro={borrando === coleccion.id} title={t("col.eliminarPista")} onclick={borrarColeccion}>
            <Trash2 size={12} strokeWidth={2} />{borrando === coleccion.id ? t("col.seguro") : t("col.eliminar")}
          </button>
        {/if}
      </div>
    </div>

    {#if detalleF}
      <Especimen
        f={detalleF} inicial={estiloInicial}
        {muestra}
        fav={favs.has(detalleF.nombre)}
        bloqueada={bloqueada(detalleF)}
        volver={seccion === "favoritas" ? t("favorites") : coleccion ? coleccion.nombre : t("sec.instaladas")}
        onvolver={() => (detalle = null)}
        onestilo={s => alternarUna({...detalleF, estiloVista:s, activa:s.activa})}
        onalternar={() => alternarUna(detalleF)}
        onfav={() => alternarFavorita(detalleF.nombre)}
      />
    {:else if comparadas.length}
      <Comparar
        familias={comparadas}
        {muestra}
        {tamano}
        onvolver={() => (comparar = [])}
        onquitar={(n) => (comparar = comparar.length <= 2 ? [] : comparar.filter((x) => x !== n))}
      />
    {:else if enDescubrir}
      {#if descubrir.lista.length && listaGoogle.length}
        <ListaVirtual items={listaGoogle} alto={Math.ceil(34 + tamano * 1.58)} clave={(g) => g.nombre} fila={filaGoogle} alVer={alVerGoogle} padBottom={60} />
      {:else if descubrir.lista.length}
        <Vacio titulo={t("vacio.nada", { q: q.trim() })} texto={t("vacio.nadaGoogle")} boton={t("vacio.borrar")} onboton={() => (q = "")} />
      {:else if descubrir.error}
        <Vacio icono={WifiOff} titulo={t("g.sinRed")} texto={descubrir.error} boton={t("g.reintentar")} onboton={() => cargarCatalogo(true)} />
      {:else}
        <Esqueleto />
      {/if}
    {:else if fuentes.cargando}
      {#if fuentes.esqueleto}<Esqueleto />{/if}
    {:else if !listaVista.length}
      {#if cantidad(filtros) && seccion!=="activas" && seccion!=="inactivas"}
        <Vacio titulo={t("vacio.lista")} boton={t("filter.clear")} onboton={()=>filtros=vacios()}/>
      {:else if seccion === "favoritas" && !ql && !base.length && !biblioteca.favoritas.some((n) => descubrir.lista.some((g) => g.nombre === n))}
        <Vacio icono={Heart} titulo={t("vacio.favTitulo")} texto={t("vacio.favTexto")} boton={t("vacio.favBoton")} onboton={() => irA("instaladas")} />
      {:else if ql}
        <Vacio
          titulo={t("vacio.nada", { q: q.trim() })}
          texto={t("vacio.nadaTexto")}
          boton={t("vacio.buscarGoogle")}
          onboton={() => {
            const b = q;
            irA("descubrir");
            q = b;
          }}
        />
      {:else if carpeta && leyendo === carpeta}
        <Vacio icono={LoaderCircle} girando titulo={t("vacio.leyendo")} texto={t("vacio.leyendoTexto")} />
      {:else}
        <Vacio icono={coleccion ? Layers : carpeta ? FolderOpen : Type} titulo={t("vacio.lista")} texto={coleccion ? t("vacio.colTexto") : carpeta ? t("vacio.carTexto") : ""} />
      {/if}
    {:else if ajustes.vista === "cuadricula"}
      {#key seccion + q + JSON.stringify(filtros)}
        <ListaVirtual items={listaVista} alto={146} minAncho={176} gap={10} padX={24} padTop={4} clave={(f) => f.claveVista ?? JSON.stringify([!!f.google, f.nombre])} fila={tarjetaFav} alVer={alVerFav} />
      {/key}
    {:else}
      {#key seccion + q + JSON.stringify(filtros)}
        <ListaVirtual items={listaVista} alto={Math.ceil(57 + tamano * 1.58)} clave={(f) => f.claveVista ?? JSON.stringify([!!f.google, f.nombre])} fila={filaFav} alVer={alVerFav} padBottom={80} />
      {/key}
    {/if}

    {#if seleccionadas.length && vistaNormal && !enDescubrir}
      <BarraSeleccion
        n={seleccionadas.length}
        colecciones={biblioteca.colecciones}
        bind:abierta={colAbierta}
        onactivar={() => alternarVarias(seleccionadas, true)}
        ondesactivar={() => alternarVarias(seleccionadas, false)}
        onfavoritas={() => {
          const n = familiasSeleccionadas;
          aFavoritas(n);
          sel.clear();
          avisar("ok", t("ok.aFavoritas", { n: n.length }), "");
        }}
        oncoleccion={(id) => {
          aCol(id, familiasSeleccionadas);
          sel.clear();
          colAbierta = false;
        }}
        oncomparar={() => {
          if (familiasSeleccionadas.length < 2 || familiasSeleccionadas.length > 4) return;
          comparar = familiasSeleccionadas;
          sel.clear();
        }}
        onlimpiar={() => sel.clear()}
      />
    {/if}

    {#if soltando}
      <div class="soltar">
        <div class="marco">
          <img src="/icon.svg" width="92" height="92" alt="" draggable="false" />
          <span class="st">{t("dropTitle")}</span>
          <span class="sx">{t("dropText")}</span>
        </div>
      </div>
    {/if}
  </main>

  {#if arrastre.activo}
    <div class="fantasma" style:left="{arrastre.x + 12}px" style:top="{arrastre.y + 10}px">
      {arrastre.nombres.length === 1 ? arrastre.nombres[0] : t("fuentesN", { n: arrastre.nombres.length })}
    </div>
  {/if}

  {#if ajustesEn}
    <Ajustes {carpetas} onanadirCarpeta={() => anadirCarpeta()} onquitarCarpeta={quitarCarpeta} onelegirCarpeta={() => anadirCarpeta(true)} bind:pestana={ajustesEn} oncerrar={() => (ajustesEn = null)} cuentas={{ instaladas: instaladas.length, activas: activasN, favoritas: biblioteca.favoritas.length }} />
  {/if}

  <Avisos />

  {#if splash}
    <div class="splash">
      <Marca ancho={245} />
    </div>
  {/if}
</div>

<style>
  .grupo-personal { display:flex; flex-direction:column; align-items:flex-start; justify-content:center; gap:4px; margin:4px 12px; padding:8px; text-align:left; color:var(--ink); background:var(--field); border-radius:6px; }
  .grupo-personal small { color:var(--ink2); }
  .ventana { position: relative; height: calc(100dvh / var(--ui-scale,1)); overflow: hidden; display: grid; grid-template-columns: 216px minmax(0, 1fr); }
  main { position: relative; display: flex; flex-direction: column; min-width: 0; min-height: 0; }
  header { flex: none; height: 52px; display: flex; align-items: center; gap: 10px; padding-left: 24px; }
  .hueco { flex: 1; align-self: stretch; }
  .campo {
    height: 28px;
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 0 9px;
    border-radius: 6px;
    background: var(--field);
    box-shadow: 0 0 0 1px var(--line);
    color: var(--ink3);
  }
  .buscar { flex: 0 0 190px; }
  .prueba { flex: 1 1 auto; min-width: 0; max-width: 360px; }
  .campo input { flex: 1; min-width: 0; border: 0; outline: 0; background: transparent; font: 500 12.5px var(--sans); color: var(--ink); }
  .a { font: 700 12px var(--sans); color: var(--ink3); }
  .reset { width: 16px; height: 16px; border: 0; border-radius: 50%; background: var(--line2); color: var(--win); padding: 0; display: flex; align-items: center; justify-content: center; flex: none; }
  .tamano { flex: none; display: flex; align-items: center; gap: 8px; margin-left: 8px; }
  .tamano .pq { font: 600 10px var(--sans); }
  .tamano .gr { font: 600 15px var(--sans); }
  .tamano .n { width: 22px; font-size: 11.5px; color: var(--ink3); }
  .rango { -webkit-appearance: none; appearance: none; width: 90px; height: 2px; background: var(--line2); border-radius: 2px; outline: none; }
  .rango::-webkit-slider-thumb { -webkit-appearance: none; width: 12px; height: 12px; border-radius: 50%; background: var(--ink); border: 0; cursor: pointer; }
  .vista { margin-right: 12px; display: flex; }

  .titulo { flex: none; display: flex; align-items: center; gap: 8px; padding: 10px 24px 8px; min-height: 46px; }
  .cuenta { font-size: 10.5px; color: var(--ink3); }
  .derecha { margin-left: auto; display: flex; align-items: center; gap: 2px; }
  .sep { width: 1px; height: 16px; background: var(--line2); margin: 0 6px; }
  .pild {
    height: 24px;
    padding: 0 10px;
    border: 0;
    border-radius: 999px;
    background: var(--chip);
    color: var(--ink);
    font: 600 11.5px var(--sans);
    display: flex;
    align-items: center;
    gap: 5px;
    flex: none;
    transition: background-color 0.15s, color 0.15s;
  }
  .pild.tinta { background: var(--ink); color: var(--win); }
  .pild.borrar { background: transparent; color: var(--ink2); padding: 0 10px 0 8px; }
  .pild.borrar:hover { background: var(--hov); }
  .pild.borrar.seguro { background: var(--red); color: #fff; }

  .soltar { position: absolute; inset: 0; z-index: 9; padding: 14px; background: var(--win); animation: tf-fade 0.15s ease-out; }
  .marco { height: 100%; border: 1.5px dashed var(--line2); border-radius: 12px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; text-align: center; }
  .marco img { display: block; }
  .st { font-size: 17px; font-weight: 700; }
  .sx { font-size: 13.5px; color: var(--ink2); }

  .fantasma {
    position: fixed;
    z-index: 50;
    pointer-events: none;
    padding: 6px 10px;
    border-radius: 999px;
    background: var(--ink);
    color: var(--win);
    font: 600 12px var(--sans);
    white-space: nowrap;
  }

  main {container-type:inline-size;}
  @container (max-width: 760px) {
    header {padding-left:12px;gap:6px;flex-wrap:wrap;height:auto;min-height:48px;}
    .buscar {flex:1 1 110px;min-width:85px;}
    .prueba {flex:1 1 120px;}
    .rango {width:52px;}
    .titulo {flex-wrap:wrap;padding-inline:12px;}
    .derecha {flex-wrap:wrap;}
  }
  .splash { position: absolute; inset: 0; z-index: 40; background: var(--win); display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 22px; }
</style>
