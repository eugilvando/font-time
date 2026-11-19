import test from "node:test";
import assert from "node:assert/strict";
import {coincideEstilo,distribucion,vacios} from "./filtros.js";
test("Intersect state, metadata and ranges on the same style",()=>{
  const a={activa:false,duplicada:true,propiedades:{variable:true,color:true,peso:700,contraste:0}};
  const b={activa:true,duplicada:false,propiedades:{peso:400,contraste:null}};
  const filtros={...vacios(),estado:"inactivas",variable:true,color:true,duplicada:true,rangos:{peso:[650,750],contraste:[0,0]}};
  assert.ok(coincideEstilo(a,filtros));assert.ok(!coincideEstilo(b,filtros));
  assert.ok(!coincideEstilo(b,{...vacios(),rangos:{contraste:[0,10]}}));
  assert.equal(distribucion([a,b],"contraste").count,1);
  assert.equal(distribucion([a,b],"peso").bins.reduce((a,n)=>a+n,0),2);
});
