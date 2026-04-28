import type { UserPreferences } from "../types/settings";

export type AppTheme = UserPreferences["theme"];

const THEME_STORAGE_KEY = "cdrive-cleaner-theme";

function resolveTheme(theme: AppTheme): "dark" | "light" {
  if (theme === "system") {
    return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
  }

  return theme;
}

export function applyTheme(theme: AppTheme) {
  const resolvedTheme = resolveTheme(theme);
  document.documentElement.dataset.theme = resolvedTheme;
  document.documentElement.style.colorScheme = resolvedTheme;
  localStorage.setItem(THEME_STORAGE_KEY, theme);
}

export function getStoredTheme(): AppTheme {
  const saved = localStorage.getItem(THEME_STORAGE_KEY);
  if (saved === "light" || saved === "dark" || saved === "system") {
    return saved;
  }

  return "dark";
}

export function bootstrapTheme() {
  applyTheme(getStoredTheme());
}
