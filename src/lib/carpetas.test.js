import test from "node:test";
import assert from "node:assert/strict";
import {arbolCarpetas} from "./carpetas.js";

test("Folder indicators follow their own styles and nested folders",()=>{
  const root="/OneDrive/Fonts";
  const regular=root+"/A/regular.ttf", bold=root+"/B/bold.ttf";
  const f={activa:true,archivos:[regular,bold],estilos:[
    {activa:false,fuentes:[regular]}, {activa:true,fuentes:[bold],archivo:"/active/bold.ttf"}
  ]};
  const tree=()=>arbolCarpetas([{ruta:root}],[],[f])[0];
  let r=tree();
  assert.deepEqual([r.cuenta,r.activas,r.totalEstilos],[1,1,2]);
  assert.deepEqual(r.hijos.map(c=>[c.activas,c.totalEstilos]),[[0,1],[1,1]]);
  f.estilos[0].activa=true;
  assert.equal(tree().activas,2);
  f.estilos.forEach(e=>e.activa=false);
  assert.equal(tree().activas,0);
});
