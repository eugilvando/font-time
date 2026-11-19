export const categorias = ["sans","serif","slab","script","display","blackletter","mono","symbols","unknown","otros"];
export const medidas = ["peso","contraste","proporcion","altura"];
export const vacios = () => ({estado:"todas", categorias:[], favoritas:false, variable:false, color:false, duplicada:false, etiqueta:"", rangos:{}});
export function cantidad(f) {
  return f.categorias.length + Number(f.estado!=="todas") + Number(f.favoritas) + Number(f.variable) + Number(f.color) + Number(f.duplicada) + Number(!!f.etiqueta) + Object.keys(f.rangos).length;
}
export const valor = (s,k) => k==="peso" ? (s.propiedades ? s.propiedades.peso : s.peso) : s.propiedades?.[k];
export function coincideEstilo(s,f) {
  if(f.estado!=="todas" && (f.estado==="activas")!==!!s.activa) return false;
  if(f.variable && !s.propiedades?.variable || f.color && !s.propiedades?.color || f.duplicada && !s.duplicada) return false;
  return Object.entries(f.rangos).every(([k,[min,max]])=>{
    const n=valor(s,k);
    return Number.isFinite(n) && n>=min && n<=max;
  });
}
export function distribucion(estilos,k) {
  const valores=estilos.map(s=>valor(s,k)).filter(Number.isFinite);
  const min=k==="peso"?1:0;
  const max=valores.reduce((n,v)=>Math.max(n,v),k==="peso"?1000:100);
  const bins=Array(24).fill(0);
  for(const v of valores) bins[Math.min(23,Math.max(0,Math.floor(24*(v-min)/(max-min))))]++;
  return {min,max,bins,count:valores.length};
}
