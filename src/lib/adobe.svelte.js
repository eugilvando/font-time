import { ajustes, enTauri, plataforma } from "./ajustes.svelte.js";
import { llamar, actualizarFuentes, fuentes } from "./fuentes.svelte.js";
export const adobe=$state({ connected:false,owned:[],error:"" });
let ocupado=false;
export async function mantener(nombre) { await llamar("adobe_mantener",{familia:nombre}); await consultar(); }
async function consultar() {
  if (!ajustes.adobe || !enTauri || plataforma==="linux" || ocupado || fuentes.ocupado || fuentes.cargando) return;
  ocupado=true;
  try {
    const r=await llamar("adobe_actualizar",{cerrar:ajustes.adobeCerrar});
    adobe.connected=r.connected; adobe.owned=r.owned; adobe.error=r.errors.join("\n");
    if(r.changed) await actualizarFuentes();
  } catch(e) {adobe.error=String(e);} finally {ocupado=false;}
}
export function iniciarAdobe() {
  const timer=setInterval(consultar,5000);
  return () => clearInterval(timer);
}
