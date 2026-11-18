import en from "./en.js";
import es from "./es.js";
import ptBR from "./pt-BR.js";

export const textos = { en, es, "pt-BR": ptBR };
export const locales = { en: "en-US", es: "es-ES", "pt-BR": "pt-BR" };
export const idiomaValido = (valor) => Object.hasOwn(textos, valor) ? valor : "en";
