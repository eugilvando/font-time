const canon = s => s.trim().toLowerCase();
const nombre = s => typeof s === "string" && s.trim().length > 0 && s.length <= 1024;

export function leerColecciones(contenido) {
  const doc = JSON.parse(contenido);
  if (doc.format !== "font-time-collections" || doc.version !== 1 || !Array.isArray(doc.collections)) throw new Error("Font Time Collections v1");
  return doc.collections.map(c => {
    if (!nombre(c.name) || !Array.isArray(c.families) || !c.families.every(nombre)) throw new Error("Invalid collection");
    return { nombre:c.name.trim(), fuentes:[...new Set(c.families.map(f => f.trim()))] };
  });
}
export function exportar(lista) {
  return JSON.stringify({ format:"font-time-collections", version:1,
    collections:lista.map(c => ({ name:c.nombre, families:[...c.fuentes] })) }, null, 2);
}
export function fusionar(actual, entrada) {
  const copia = actual.map(c => ({ ...c, fuentes:[...c.fuentes] }));
  for (const c of entrada) {
    const otra = copia.find(v => canon(v.nombre) === canon(c.nombre));
    if (!otra) copia.push({ id:crypto.randomUUID(), nombre:c.nombre, fuentes:[...new Set(c.fuentes)] });
    else {
      const vistas = new Set(otra.fuentes.map(canon));
      for (const f of c.fuentes) if (!vistas.has(canon(f))) { vistas.add(canon(f)); otra.fuentes.push(f); }
    }
  }
  return copia;
}

// A journal entry changes a single field or membership. Deleting a collection
// leaves its existence flag in the journal, so an offline rename cannot revive it.
function campos(colecciones) {
  const keys = new Map();
  for (const c of colecciones) {
    keys.set(JSON.stringify([c.id,"exists"]),true);
    keys.set(JSON.stringify([c.id,"name"]),c.nombre);
    for (const f of c.fuentes) keys.set(JSON.stringify([c.id,"font",f]),true);
  }
  return keys;
}
export function cambios(antes, despues) {
  const old = campos(antes), cur = campos(despues);
  const ops = [];
  for (const [key,v] of cur) if (v !== old.get(key)) ops.push([...JSON.parse(key),v]);
  for (const key of old.keys()) {
    const [id,tipo,f] = JSON.parse(key);
    if (cur.has(key)) continue;
    if (tipo === "exists") ops.push([id,tipo,false]);
    if (tipo === "font" && cur.has(JSON.stringify([id,"exists"]))) ops.push([id,tipo,f,false]);
  }
  return ops;
}
// A cloud folder can briefly list only a subset of its files. A missing
// journal batch is never a user deletion (deletions have explicit operations).
export function diarioCompleto(diario, confirmados) {
  const presentes = new Set(diario.map(lote => lote.id));
  return confirmados.every(id => presentes.has(id));
}

export function reproducir(diario) {
  const docs = new Map();
  const orden = [...diario].sort((a,b) => a.clock - b.clock || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
  let clock = 0;
  for (const d of orden) {
    if (d.version !== 1 || !nombre(d.id) || !Number.isSafeInteger(d.clock) || d.clock < 1 || !Array.isArray(d.ops)) throw new Error("Invalid sync journal");
    clock = Math.max(clock,d.clock);
    for (const op of d.ops) {
      if (!Array.isArray(op) || !nombre(op[0])) throw new Error("Invalid sync operation");
      const [id,tipo,v,activo] = op;
      if (!docs.has(id)) docs.set(id,{ id, existe:false, nombre:"", fuentes:new Set() });
      const c = docs.get(id);
      if (tipo === "exists" && typeof v === "boolean") c.existe = v;
      else if (tipo === "name" && nombre(v)) c.nombre = v;
      else if (tipo === "font" && nombre(v) && typeof activo === "boolean") { if (activo) c.fuentes.add(v); else c.fuentes.delete(v); }
      else throw new Error("Unknown sync operation");
    }
  }
  return { clock, lista:[...docs.values()].filter(c => c.existe && c.nombre).map(c => ({id:c.id,nombre:c.nombre,fuentes:[...c.fuentes]})) };
}
