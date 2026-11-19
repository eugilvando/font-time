import { ajustes, enTauri } from "./ajustes.svelte.js";
import { t } from "./i18n.svelte.js";
import { llamar } from "./fuentes.svelte.js";

export function muestrasActuales() {
  if (Array.isArray(ajustes.muestras)) return ajustes.muestras;
  return [ajustes.textoPrueba.trim() || t("defaultSample"), ...t("org.sampleMore").split("|")];
}

export function grupoDe(nombre) {
  return ajustes.grupos.find(x => x.nombre === nombre)?.valor ?? "";
}
export function configurar(clave, nombre, valor) {
  const otras = ajustes[clave].filter(x => x.nombre !== nombre);
  ajustes[clave] = valor.length ? [...otras, { nombre, valor }] : otras;
}

const publicas = $state({ porFamilia: {} });
$effect.root(() => {
  $effect(() => {
    if (!ajustes.tagsPublicos || !enTauri) return;
    let activo = true;
    llamar("etiquetas_publicas").then(mapa => { if (activo) publicas.porFamilia = mapa; }).catch(() => {});
    return () => { activo = false; };
  });
});
export function etiquetasDe(f) {
  const personal = ajustes.etiquetas.find(x => x.nombre === f.nombre)?.valor ?? [];
  const sistema = ajustes.tagsSistema ? [t("cat." + f.categoria), ...f.estilos.map(e => e.nombre)] : [];
  const remotas = publicas.porFamilia[f.nombre];
  const comunidad = ajustes.tagsPublicos && Array.isArray(remotas) ? remotas : [];
  return { personal, sistema: [...new Set(sistema)], comunidad: [...new Set(comunidad)] };
}

export function palabrasDe(f) {
  const { personal, sistema, comunidad } = etiquetasDe(f);
  return [...new Set([...personal, ...sistema, ...comunidad])];
}

export function filasDeFamilias(familias, abiertos) {
  const grupos = new Map(ajustes.grupos.map(x => [x.nombre, x.valor]));
  const filas = familias.flatMap(f => {
    if (f.google) return [f];
    const grupo = grupos.get(f.nombre);
    if (ajustes.agrupar || !f.estilos.length) {
      return [grupo ? { ...f, tituloVista: grupo + " · " + f.nombre } : f];
    }
    return f.estilos.map((estilo) => ({
      ...f, activa: !!estilo.activa, estiloVista: estilo,
      claveVista: JSON.stringify([f.nombre, estilo.nombre, estilo.peso, estilo.cursiva]),
      tituloVista: (grupo ? grupo + " · " : "") + f.nombre + " · " + estilo.nombre,
    }));
  });
  if (grupos.size) filas.sort((a,b) => (a.tituloVista || a.nombre).localeCompare(b.tituloVista || b.nombre));
  if (!ajustes.agrupar || !grupos.size) return filas;
  const miembros = new Map();
  for (const fila of filas) {
    const grupo = !fila.google && grupos.get(fila.nombre);
    if (!grupo) continue;
    if (!miembros.has(grupo)) miembros.set(grupo, []);
    miembros.get(grupo).push(fila);
  }
  const vistos = new Set();
  return filas.flatMap(fila => {
    const grupo = !fila.google && grupos.get(fila.nombre);
    if (!grupo) return [fila];
    if (vistos.has(grupo)) return [];
    vistos.add(grupo);
    const hijos = miembros.get(grupo);
    const cabecera = { grupo, cantidad:hijos.length, nombre:grupo, claveVista:JSON.stringify(["group",grupo,0]) };
    return abiertos.has(grupo) ? [cabecera,...hijos] : [cabecera];
  });
}
