// t("clave", { var }) en cualquier componente. Lee ajustes.idioma, así que lo
// que lo use se repinta solo al cambiar de idioma.
import { ajustes } from "./ajustes.svelte.js";
import { textos, idiomaValido, locales } from "../i18n/idiomas.js";

export const idioma = () => idiomaValido(ajustes.idioma);

export function t(clave, vars) {
  let texto = textos[idioma()][clave] ?? textos.en[clave] ?? clave;
  if (vars) for (const [k, v] of Object.entries(vars)) texto = texto.replaceAll(`{${k}}`, v);
  return texto;
}

export const locale = () => locales[idioma()];

$effect.root(() => {
  $effect(() => {
    document.documentElement.lang = idioma();
  });
});
