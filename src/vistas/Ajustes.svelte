<script>
  import Marca from "../componentes/Marca.svelte";
  // Modal de ajustes con tres pestañas: ajustes · atajos · acerca de.
  import { SlidersHorizontal, Keyboard, Info, X, ArrowUpRight } from "@lucide/svelte";
  import Segmentado from "../componentes/Segmentado.svelte";
  import { ajustes, ponerTema, ponerIdioma, version, ATAJOS } from "../lib/ajustes.svelte.js";
  import { t } from "../lib/i18n.svelte.js";
  import { avisar } from "../lib/avisos.svelte.js";
  import { respaldadas, restaurarSistema, fuentes } from "../lib/fuentes.svelte.js";
  import { comboDe } from "../lib/atajos.js";
  import { nueva, buscarApp, actualizadorDisponible } from "../lib/actualizador.svelte.js";
  import { enTauri, esWindows, plataforma } from "../lib/ajustes.svelte.js";

  import BibliotecaAjustes from "./BibliotecaAjustes.svelte";
  const abrirWeb = async (cual) => {
    if (!enTauri) return;
    const { invoke } = await import("@tauri-apps/api/core");
    invoke("abrir_web", { cual }).catch(() => {});
  };
  const estadoAct = $derived(
    nueva.estado === "buscando"
      ? t("act.buscando")
      : nueva.estado === "nada"
        ? t("act.alDia")
        : nueva.estado === "available" || nueva.estado === "downloading" || nueva.estado === "installing"
          ? t("act.hay", { v: nueva.version })
          : nueva.estado === "error"
            ? t("act.sinRespuesta")
            : t("aj.actHint"),
  );

  let { pestana = $bindable(), oncerrar, cuentas, carpetas, onanadirCarpeta, onquitarCarpeta, onelegirCarpeta } = $props();

  const PESTANAS = [
    { id: "ajustes", texto: () => t("settings"), icono: SlidersHorizontal },
    { id: "atajos", texto: () => t("shortcuts"), icono: Keyboard },
    { id: "acerca", texto: () => t("about"), icono: Info },
  ];

  // Abrir al iniciar Windows: lo que diga el registro, no un ajuste guardado
  // (se puede apagar desde el Administrador de tareas).
  let inicioWin = $state(false);
  if (enTauri && esWindows) {
    import("@tauri-apps/api/core").then(({ invoke }) => invoke("inicio_windows").then((v) => (inicioWin = v)).catch(() => {}));
  }
  async function ponerInicio(v) {
    if (!enTauri) return;
    const { invoke } = await import("@tauri-apps/api/core");
    try {
      inicioWin = await invoke("poner_inicio_windows", { activar: v });
      if (inicioWin !== v) avisar("err", t("err.inicio"), "");
    } catch (e) {
      avisar("err", t("err.inicio"), String(e));
    }
  }

  let nRespaldo = $state(0);
  if (esWindows) respaldadas().then((n) => (nRespaldo = n)).catch(() => {});

  function desbloquear(v) {
    ajustes.desbloqueadas = v;
    if (v) avisar("warn", t("aviso.desbloqueadas"), t("aviso.desbloqueadasTexto"));
    else avisar("ok", t("ok.bloqueadas"), t("ok.bloqueadasTexto"));
  }

  async function restaurar() {
    const r = await restaurarSistema();
    nRespaldo = await respaldadas().catch(() => 0);
    if (r && r.cambiadas) avisar("ok", t("ok.restauradas"), r.reiniciar ? t("aviso.reiniciarTexto") : "");
  }

  const filas = $derived([
    { etiqueta: t("aj.tema"), pista: t("aj.temaHint"), valor: ajustes.tema, elegir: ponerTema, opciones: [[null, t("aj.temaSistema")], ["light", t("aj.claro")], ["dark", t("aj.oscuro")]] },
    { etiqueta: t("aj.abrir"), pista: t("aj.abrirHint"), valor: ajustes.alAbrir, elegir: (v) => (ajustes.alAbrir = v), opciones: [["instaladas", t("sec.instaladas")], ["favoritas", t("sec.favoritas")], ...(ajustes.carpetaPrincipal ? [["carpeta", t("mainFolder")]] : [])] },
    ...(esWindows ? [
    { etiqueta: t("aj.cerrar"), pista: t("aj.cerrarHint"), valor: ajustes.alCerrar, elegir: (v) => (ajustes.alCerrar = v), opciones: [["bandeja", t("aj.bandeja")], ["salir", t("aj.salir")]] },
    { etiqueta: t("aj.inicio"), pista: t("aj.inicioHint"), valor: inicioWin, elegir: ponerInicio, opciones: [[true, t("aj.si")], [false, t("aj.no")]] },
    { etiqueta: t("aj.sistema"), pista: t("aj.sistemaHint"), valor: ajustes.desbloqueadas, elegir: desbloquear, opciones: [[false, t("aj.bloqueadas")], [true, t("aj.desbloquear")]] },
    ] : []),
    { etiqueta: t("aj.ver"), pista: t("aj.verHint"), valor: ajustes.verBloqueadas, elegir: (v) => (ajustes.verBloqueadas = v), opciones: [[false, t("aj.ocultas")], [true, t("aj.visibles")]] },
    ...(actualizadorDisponible ? [{ etiqueta: t("aj.buscarAct"), pista: t("aj.buscarActHint"), valor: ajustes.buscarActualizaciones, elegir: (v) => (ajustes.buscarActualizaciones = v), opciones: [[true, t("aj.si")], [false, t("aj.no")]] }] : []),
    { etiqueta: t("aj.idioma"), pista: "", valor: ajustes.idioma, elegir: ponerIdioma, opciones: [["pt-BR", "Português (Brasil)"], ["en", "English"], ["es", "Español"]] },
  ]);

  // ---- atajos ----
  let grabando = $state(null);
  const bonita = (k) => (k === "Space" ? t("atajos.espacio") : k);
  const atajos = $derived([
    { id: "buscar", que: t("atajos.buscar") },
    { id: "todas", que: t("atajos.todas") },
    { id: "alternar", que: t("atajos.alternar") },
    { fijo: t("atajos.dobleClic"), que: t("atajos.doble") },
    { fijo: "Esc", que: t("atajos.esc") },
  ]);
  const cambiados = $derived(JSON.stringify(ajustes.atajos) !== JSON.stringify(ATAJOS));

  function tecla(e) {
    if (grabando) {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") return (grabando = null);
      const c = comboDe(e);
      if (!c) return;
      const choca = Object.keys(ajustes.atajos).find((k) => k !== grabando && ajustes.atajos[k] === c);
      const antes = ajustes.atajos[grabando];
      ajustes.atajos = { ...ajustes.atajos, [grabando]: c, ...(choca ? { [choca]: antes } : {}) };
      grabando = null;
      avisar("ok", t("ok.atajo", { c: c.split("+").map(bonita).join("+") }), choca ? t("ok.atajoCambio") : "");
      return;
    }
    if (e.key === "Escape") {
      e.preventDefault();
      oncerrar();
    }
  }

  function resetear() {
    ajustes.atajos = { ...ATAJOS };
    grabando = null;
    avisar("ok", t("ok.atajosRestaurados"), "");
  }

  const novedades = $derived([t("nov.library"), t("nov.collections"), t("nov.adobe")]);
</script>

<svelte:window onkeydowncapture={tecla} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="velo" onclick={oncerrar}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="modal" role="dialog" aria-modal="true" aria-label={t("settings")} tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <div class="lado">
      <div class="marca-lado"><Marca ancho={103} /></div>
      {#each PESTANAS as p (p.id)}
        <button type="button" class="tab" class:activa={pestana === p.id} onclick={() => ((pestana = p.id), (grabando = null))}>
          <p.icono size={14} strokeWidth={1.9} />{p.texto()}
        </button>
      {/each}
    </div>

    <div class="cuerpo">
      <!-- Título y cerrar quedan fijos; solo se desplaza lo de debajo. -->
      <div class="cab">
        <h2>{pestana === "ajustes" ? t("settings") : pestana === "atajos" ? t("shortcuts") : t("about")}</h2>
        <button type="button" class="cerrar" title={t("close")} aria-label={t("close")} onclick={oncerrar}>
          <X size={13} strokeWidth={2} />
        </button>
      </div>

      <div class="scroll">
      {#if pestana === "ajustes"}
        <div class="fila">
          <span class="txt"><span class="eti">{t("mainFolder")}</span><span class="pista ruta" title={ajustes.carpetaPrincipal}>{ajustes.carpetaPrincipal || t("mainFolderHint")}</span></span>
          <button type="button" class="boton" onclick={onelegirCarpeta}>{t("chooseFolder")}</button>
        </div>
        <section class="carpetas" aria-label={t("grupo.carpetas")}>
          <div class="fila">
            <div class="txt"><span class="eti">{t("grupo.carpetas")}</span><span class="pista">{t("folderInfo")}</span></div>
            <button type="button" class="boton" onclick={onanadirCarpeta}>{t("car.anadir")}</button>
          </div>
          {#each carpetas as c (c.ruta)}
            <div class="fila carpeta" data-ruta={c.ruta}>
              <span class="pista ruta" title={c.ruta}>{c.ruta}</span>
              <button type="button" class="boton" onclick={() => onquitarCarpeta(c.ruta)}>{t("stopWatching")}</button>
            </div>
          {/each}
        </section>
        {#each filas as f (f.etiqueta)}
          <div class="fila">
            <span class="txt">
              <span class="eti">{f.etiqueta}</span>
              {#if f.pista}<span class="pista">{f.pista}</span>{/if}
            </span>
            <Segmentado etiqueta={f.etiqueta} opciones={f.opciones.map(([valor, etiqueta]) => ({ valor, etiqueta }))} valor={f.valor} onelegir={f.elegir} />
          </div>
        {/each}
        <div class="fila">
          <label class="txt" for="desactivar-salida">
            <span class="eti">{t("aj.desactivarSalir")}</span>
            <span class="pista">{t("aj.desactivarSalirHint")}</span>
          </label>
          <input id="desactivar-salida" class="toggle" role="switch" type="checkbox" bind:checked={ajustes.desactivarAlSalir} />
        </div>
        {#if actualizadorDisponible}
          <div class="fila">
            <span class="txt">
              <span class="eti">{t("aj.act")}</span>
              <span class="pista">{estadoAct}</span>
            </span>
            {#if nueva.estado === "available" && !nueva.cerrado}
              <button type="button" class="boton" onclick={oncerrar}>{t("act.verAviso")}</button>
            {:else}
              <button type="button" class="boton" disabled={nueva.estado === "buscando" || !enTauri} onclick={() => buscarApp(true)}>{t("act.buscarAhora")}</button>
            {/if}
          </div>
        {/if}
        {#if nRespaldo > 0}
          <div class="fila">
            <span class="txt">
              <span class="eti">{t("aj.restaurar")}</span>
              <span class="pista">{t("aj.restaurarHint", { n: nRespaldo })}</span>
            </span>
            <button type="button" class="boton" disabled={fuentes.ocupado} onclick={restaurar}>{t("aj.restaurar")}</button>
          </div>
        {/if}
        <BibliotecaAjustes />
      {:else if pestana === "atajos"}
        <span class="pista menos">{t("atajos.hint")}</span>
        <div class="atajos">
          {#each atajos as a (a.que)}
            {#if a.id}
              <button type="button" class="teclas" class:grabando={grabando === a.id} title={t("atajos.cambiar")} onclick={() => (grabando = grabando === a.id ? null : a.id)}>
                {#if grabando === a.id}
                  <kbd>{t("atajos.pulsa")}</kbd>
                {:else}
                  {#each ajustes.atajos[a.id].split("+") as k (k)}<kbd>{bonita(k)}</kbd>{/each}
                {/if}
              </button>
            {:else}
              <span class="teclas fijo" title={t("atajos.fijo")}><kbd>{a.fijo}</kbd></span>
            {/if}
            <span class="que">{a.que}</span>
          {/each}
        </div>
        {#if cambiados}
          <button type="button" class="boton suelto" onclick={resetear}>{t("atajos.reset")}</button>
        {/if}
      {:else}
        <div class="acerca">
          <div class="nombre">
            <Marca ancho={158} />
            <span class="ver">{esWindows ? t("version", { v: version }) : version + " · " + (plataforma === "macos" ? "macOS" : "Ubuntu")}</span>
          </div>
        </div>
        <p class="tagline">{t("tagline")}</p>
        <div class="numeros">
          {#each [[cuentas.instaladas, t("acerca.instaladas")], [cuentas.activas, t("acerca.activas")], [cuentas.favoritas, t("acerca.favoritas")]] as [n, l] (l)}
            <div class="dato"><span class="n num">{n}</span><span class="l">{l}</span></div>
          {/each}
        </div>
        <div class="nov">
          <span class="etiqueta">{t("whatsNew", { v: version })}</span>
          {#each novedades as n (n)}
            <div class="linea"><span class="punto"></span><span>{n}</span></div>
          {/each}
        </div>
        <div class="enlaces">
          <div class="enlace">
            <span>{t("acerca.proyecto")}</span>
            <button type="button" class="link" onclick={() => abrirWeb("repo")}>GitHub<ArrowUpRight size={12} strokeWidth={2} /></button>
          </div>
          <div class="enlace">
            <span>{t("aj.act")}</span>
            <button type="button" class="link" onclick={() => abrirWeb("release")}>{t("acerca.versiones")}<ArrowUpRight size={12} strokeWidth={2} /></button>
          </div>
          <div class="enlace">
            <span class="l">{t("acerca.fallo")}</span>
            <button type="button" class="link" onclick={() => abrirWeb("fallo")}>{t("acerca.abrirFallo")}<ArrowUpRight size={12} strokeWidth={2} /></button>
          </div>
        </div>
        <div class="firma">
          <span>Font Time · eugilvando</span>
          <span class="copy">{t("acerca.origen")}</span>
        </div>
      {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .carpeta { padding-top: 10px; }
  .carpeta .boton { flex: none; }
  .ruta { overflow-wrap: anywhere; }
  .toggle { appearance: none; width: 36px; height: 20px; padding: 2px; border: 0; border-radius: 14px; background: var(--off); cursor: pointer; }
  .toggle::before { content: ""; display: block; width: 16px; height: 16px; border-radius: 50%; background: white; }
  .toggle:checked { background: var(--ok); }
  .toggle:checked::before { transform: translateX(16px); }
  .velo { position: absolute; inset: 0; z-index: 15; background: var(--velo); display: flex; align-items: center; justify-content: center; animation: tf-fade 0.15s ease-out; }
  .modal {
    width: 680px;
    height: 600px;
    max-height: calc(100% - 40px);
    display: grid;
    grid-template-columns: 170px minmax(0, 1fr);
    border-radius: 14px;
    overflow: hidden;
    background: var(--win);
    box-shadow: 0 30px 70px rgba(0, 0, 0, 0.3), 0 0 0 1px var(--line2);
    animation: tf-up 0.22s ease-out;
    outline: none;
  }
  .lado { background: var(--side); border-right: 1px solid var(--line); padding: 16px 10px; display: flex; flex-direction: column; gap: 2px; }
  .marca-lado { padding: 0 10px 10px; }
  .tab {
    display: flex;
    align-items: center;
    gap: 9px;
    height: 30px;
    padding: 0 10px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--ink2);
    font: 500 13px var(--sans);
    text-align: left;
    transition: background-color 0.15s, color 0.15s;
  }
  .tab :global(svg) { opacity: 0.7; }
  .tab:hover { background: var(--hov); }
  .tab.activa { background: var(--chip); color: var(--ink); font-weight: 600; }
  .tab.activa :global(svg) { opacity: 1; }
  .cuerpo { min-height: 0; display: flex; flex-direction: column; }
  .cab { flex: none; display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 14px 14px 10px 26px; }
  /* La barra de desplazamiento va separada del borde redondeado y siempre en
     su sitio (gutter estable), para que se vea y no mueva el contenido. */
  .scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    scrollbar-gutter: stable;
    margin: 0 6px 10px 0;
    padding: 4px 14px 12px 26px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .scroll { scrollbar-color: var(--off) transparent; }
  /* Hijos directos de una columna con scroll: sin esto, los de alto fijo se aplastan. */
  .scroll > :global(*) { flex-shrink: 0; }
  .cerrar {
    flex: none;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 50%;
    background: var(--chip);
    color: var(--ink2);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .cerrar:hover { background: var(--line2); color: var(--ink); }
  h2 { margin: 0; font-size: 18px; font-weight: 700; letter-spacing: -0.02em; }
  .fila { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding-bottom: 14px; border-bottom: 1px solid var(--line); }
  .txt { display: flex; flex-direction: column; gap: 3px; }
  .eti { font-size: 13.5px; font-weight: 600; }
  .pista { font-size: 12px; color: var(--ink2); }
  .pista.menos { font-size: 12.5px; }
  .boton { flex: none; height: 28px; padding: 0 12px; border: 0; border-radius: 999px; background: var(--chip); color: var(--ink); font: 600 12px var(--sans); }
  .boton:hover:not(:disabled) { background: var(--ink); color: var(--win); }
  .boton.suelto { align-self: flex-start; }
  .atajos { display: grid; grid-template-columns: auto 1fr; gap: 10px 14px; font-size: 13px; color: var(--ink2); }
  .teclas { justify-self: start; display: flex; gap: 3px; padding: 4px; margin: -4px; border: 0; border-radius: 8px; background: transparent; transition: background-color 0.15s, box-shadow 0.15s; }
  button.teclas:hover { background: var(--hov); }
  .teclas.grabando { box-shadow: 0 0 0 2px var(--ink); }
  .teclas.fijo { cursor: default; }
  .que { align-self: center; }
  kbd {
    min-width: 20px;
    height: 20px;
    padding: 0 5px;
    border-radius: 5px;
    background: var(--chip);
    box-shadow: inset 0 -1px 0 var(--line2);
    font: 600 11px var(--sans);
    color: var(--ink);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .acerca { display: flex; align-items: center; gap: 16px; }
  .acerca :global(img) { display: block; }
  .nombre { display: flex; flex-direction: column; gap: 4px; }
  .ver { font: 500 12px var(--mono); color: var(--ink2); }
  .tagline { margin: 0; font-size: 14px; line-height: 1.5; color: var(--ink2); }
  .numeros { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; }
  .dato { display: flex; flex-direction: column; gap: 2px; padding: 8px 12px; border-radius: 10px; background: var(--chip); }
  .dato .n { font-size: 20px; font-weight: 700; letter-spacing: -0.03em; line-height: 1.1; }
  .dato .l { font-size: 11.5px; color: var(--ink2); }
  .nov { display: flex; flex-direction: column; gap: 6px; }
  .linea { display: flex; gap: 9px; align-items: baseline; font-size: 12.5px; line-height: 1.45; }
  .punto { flex: none; width: 5px; height: 5px; border-radius: 50%; background: var(--red); transform: translateY(-2px); }
  .enlaces { display: flex; flex-direction: column; gap: 6px; padding-top: 12px; border-top: 1px solid var(--line); }
  .enlace { display: flex; align-items: center; justify-content: space-between; gap: 12px; font-size: 12.5px; color: var(--ink2); }
  .link { display: inline-flex; align-items: center; gap: 3px; padding: 0; border: 0; background: transparent; color: var(--ink); font: 600 12.5px var(--sans); text-decoration: underline; text-decoration-color: var(--line2); text-underline-offset: 3px; }
  .link:hover { text-decoration-color: var(--ink); }
  .firma { flex: none; margin-top: auto; display: flex; flex-wrap: wrap; gap: 6px 12px; align-items: baseline; justify-content: space-between; }
  .copy { font: 400 10.5px var(--mono); color: var(--ink3); }
</style>
