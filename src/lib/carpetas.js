// Windows conserva las mayúsculas al mostrar rutas, pero compara sin distinguirlas.
const limpiar = (ruta) => ruta.replaceAll("/", "\\").replace(/\\+$/, "");
const clave = (ruta) => limpiar(ruta).toLowerCase();
export const dentro = (ruta, carpeta) => clave(ruta).startsWith(clave(carpeta) + "\\");
export const enCarpeta = (familia, carpeta) => familia.archivos.some((ruta) => dentro(ruta, carpeta));

export function arbolCarpetas(raices, directorios, familias) {
  const orden = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });
  function nodo(ruta, nombre) {
    return { ruta, nombre, hijos: [], cuenta: 0, activas: 0 };
  }
  return raices.map(({ ruta }) => {
    const base = limpiar(ruta);
    const raiz = nodo(ruta, base.split("\\").pop() || ruta);
    const indice = new Map([[clave(ruta), raiz]]);
    function recorrer(directorio, visitados) {
      const nombreCompleto = limpiar(directorio);
      if (clave(directorio) !== clave(base) && !dentro(directorio, base)) return;
      let padre = raiz;
      let prefijo = base;
      visitados?.add(padre);
      for (const nombre of nombreCompleto.slice(base.length).split("\\").filter(Boolean)) {
        prefijo += "\\" + nombre;
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
        recorrer(p.slice(0, p.lastIndexOf("\\")), visitados);
      }
      for (const carpeta of visitados) {
        carpeta.cuenta++;
        if (familia.activa) carpeta.activas++;
      }
    }
    for (const carpeta of indice.values()) carpeta.hijos.sort((a, b) => orden.compare(a.nombre, b.nombre));
    return raiz;
  });
}
