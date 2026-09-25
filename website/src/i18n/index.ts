import { tr } from "./tr";
import { en } from "./en";
import { es } from "./es";

export type Language = "tr" | "en" | "es";

export const translations = {
  tr,
  en,
  es,
};

let currentLang: Language = (localStorage.getItem("clrinf_lang") as Language) || "tr";
if (!["tr", "en", "es"].includes(currentLang)) {
  currentLang = "tr";
}

type Listener = (lang: Language) => void;
const listeners: Listener[] = [];

export function getLang(): Language {
  return currentLang;
}

export function setLang(lang: Language) {
  if (currentLang === lang) return;
  currentLang = lang;
  localStorage.setItem("clrinf_lang", lang);
  document.documentElement.lang = lang;
  listeners.forEach((fn) => fn(lang));
}

export function onLangChange(listener: Listener) {
  listeners.push(listener);
  return () => {
    const idx = listeners.indexOf(listener);
    if (idx !== -1) listeners.splice(idx, 1);
  };
}

export function t(): typeof tr {
  return translations[currentLang] || translations.tr;
}
