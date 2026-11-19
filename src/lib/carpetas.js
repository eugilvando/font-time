const separador = (ruta) => ruta.startsWith("/") ? "/" : "\\";
const limpiar = (ruta) => separador(ruta) === "/" ? ruta.replace(/\/+$/, "") : ruta.replaceAll("/", "\\").replace(/\\+$/, "");
const clave = (ruta) => separador(ruta) === "/" ? limpiar(ruta) : limpiar(ruta).toLowerCase();
export const dentro = (ruta, carpeta) => clave(ruta).startsWith(clave(carpeta) + separador(carpeta));
export const enCarpeta = (familia, carpeta) => familia.archivos.some((ruta) => dentro(ruta, carpeta));

export function arbolCarpetas(raices, directorios, familias) {
  const orden = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });
  function nodo(ruta, nombre) {
    return { ruta, nombre, hijos: [], cuenta: 0, activas: 0, totalEstilos: 0 };
  }
  return raices.map(({ ruta }) => {
    const sep = separador(ruta);
    const base = limpiar(ruta);
    const raiz = nodo(ruta, base.split(sep).pop() || ruta);
    const indice = new Map([[clave(ruta), raiz]]);
    function recorrer(directorio, visitados) {
      const nombreCompleto = limpiar(directorio);
      if (clave(directorio) !== clave(base) && !dentro(directorio, base)) return;
      let padre = raiz;
      let prefijo = base;
      visitados?.add(padre);
      for (const nombre of nombreCompleto.slice(base.length).split(sep).filter(Boolean)) {
        prefijo += sep + nombre;
        const k = clave(prefijo);
        let hijo = indice.get(k);
        if (!hijo) {
          hijo = nodo(prefijo, nombre);
          padre.hijos.push(hijo);
          indice.set(k, hijo);
        }
        visitados?.add(hijo);
        padre = hijo;
      }
    }
    for (const dir of directorios) recorrer(dir);
    for (const familia of familias) {
      const visitados = new Set();
      for (const archivo of familia.archivos) {
        if (!dentro(archivo, ruta)) continue;
        const p = limpiar(archivo);
        recorrer(p.slice(0, p.lastIndexOf(sep)), visitados);
      }
      for (const carpeta of visitados) carpeta.cuenta++;
      const estilos = familia.estilos?.length ? familia.estilos : [familia];
      for (const estilo of estilos) {
        const destinos = new Set();
        const rutas = estilo.fuentes?.length ? estilo.fuentes : [estilo.archivo];
        for (const archivo of rutas) {
          if (!archivo || !dentro(archivo,ruta)) continue;
          const p=limpiar(archivo);
          recorrer(p.slice(0,p.lastIndexOf(sep)),destinos);
        }
        for (const carpeta of destinos) {
          carpeta.totalEstilos++;
          if (estilo.activa) carpeta.activas++;
        }
      }
    }
    for (const carpeta of indice.values()) carpeta.hijos.sort((a, b) => orden.compare(a.nombre, b.nombre));
    return raiz;
  });
}
