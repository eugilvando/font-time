import test from "node:test";
import assert from "node:assert/strict";
import {cambios, reproducir, exportar, leerColecciones, fusionar, diarioCompleto} from "./colecciones.js";
const col=(id,nombre,...fuentes)=>({id,nombre,fuentes});
const batch=(id,clock,ops)=>({id,version:1,clock,ops});

test("merge preserves missing font families and existing collection identity",()=>{
  const old=[col("original","Editorial","Inter")];
  const imported=leerColecciones(exportar([col("another","editorial","inter","Fonte ausente")]));
  assert.deepEqual(fusionar(old,imported),[col("original","Editorial","Inter","Fonte ausente")]);
  assert.throws(()=>leerColecciones('{"format":"font-time-collections","version":2,"collections":[]}'));
});
test("concurrent additions, removal and rename converge independently of delivery order",()=>{
  const base=[col("c","Manual","A","B")];
  const initial=batch("a",1,cambios([],base));
  const left=batch("b",2,cambios(base,[col("c","Revista","B","C")]));
  const right=batch("c",2,cambios(base,[col("c","Manual","A","B","D")]));
  const esperado=[col("c","Revista","B","C","D")];
  assert.deepEqual(reproducir([right,initial,left]).lista,esperado);
  assert.deepEqual(reproducir([left,right,initial,initial]).lista,esperado);
});
test("a partial or empty OneDrive listing never replaces a known local library",()=>{
  const known=["laptop","desktop"];
  assert.equal(diarioCompleto([],known),false);
  assert.equal(diarioCompleto([{id:"laptop"},{id:"unrelated"}],known),false);
  assert.equal(diarioCompleto([{id:"desktop"},{id:"laptop"},{id:"new"}],known),true);
  assert.equal(diarioCompleto([],[]),true);
});

test("offline edits and replay do not resurrect deleted collections",()=>{
  const base=[col("c","A","X")];
  const log=[batch("a",1,cambios([],base)),batch("b",2,cambios(base,[])),batch("c",3,cambios(base,[col("c","B","X","Y")]))];
  assert.deepEqual(reproducir(log).lista,[]);
  assert.throws(()=>reproducir([batch("bad",NaN,[])]));
});