import { en, type MessageKey } from "./en";
import { ja } from "./ja";
import { ko } from "./ko";

export type Locale = "en" | "ko" | "ja";

export interface LanguageOption {
  value: Locale;
  nativeLabel: string;
}

const STORAGE_KEY = "praxis.desktop.locale";

const dictionaries: Record<Locale, Record<MessageKey, string>> = {
  en,
  ko,
  ja,
};

export const DEFAULT_LOCALE: Locale = "en";

export const LANGUAGE_OPTIONS: LanguageOption[] = [
  { value: "en", nativeLabel: en["settings.languageEnglish"] },
  { value: "ko", nativeLabel: ko["settings.languageKorean"] },
  { value: "ja", nativeLabel: ja["settings.languageJapanese"] },
];

export function isLocale(value: string | null | undefined): value is Locale {
  return value === "en" || value === "ko" || value === "ja";
}

export function loadLocale(): Locale {
  if (typeof localStorage === "undefined") {
    return DEFAULT_LOCALE;
  }

  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return isLocale(stored) ? stored : DEFAULT_LOCALE;
  } catch {
    return DEFAULT_LOCALE;
  }
}

export function saveLocale(locale: Locale) {
  if (typeof localStorage === "undefined") {
    return;
  }

  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // Ignore storage errors; falling back to in-memory locale is acceptable.
  }
}

function format(template: string, vars?: Record<string, string | number | null | undefined>) {
  if (!vars) return template;

  return template.replace(/\{(\w+)\}/g, (_, key: string) => {
    const value = vars[key];
    return value === null || value === undefined ? "" : String(value);
  });
}

export function translate(
  locale: Locale,
  key: MessageKey,
  vars?: Record<string, string | number | null | undefined>,
) {
  const dict = dictionaries[locale] ?? dictionaries[DEFAULT_LOCALE];
  const template = dict[key] ?? dictionaries[DEFAULT_LOCALE][key];
  return format(template, vars);
}
