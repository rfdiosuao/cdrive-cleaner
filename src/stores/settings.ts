import { defineStore } from "pinia";
import { ref } from "vue";
import type { AIConfig, UserPreferences } from "../types/settings";

export const useSettingsStore = defineStore("settings", () => {
  const aiConfig = ref<AIConfig>({
    provider: "local",
    apiKey: "",
    model: "",
    baseUrl: "",
    temperature: 0.7,
    maxTokens: 2048,
  });

  const preferences = ref<UserPreferences>({
    autoScanOnStart: false,
    scanMode: "quick",
    cleanConfirmRequired: true,
    backupBeforeClean: true,
    language: "zh-CN",
    theme: "dark",
  });

  function updateAIConfig(config: Partial<AIConfig>) {
    aiConfig.value = { ...aiConfig.value, ...config };
  }

  function updatePreferences(prefs: Partial<UserPreferences>) {
    preferences.value = { ...preferences.value, ...prefs };
  }

  return {
    aiConfig,
    preferences,
    updateAIConfig,
    updatePreferences,
  };
});
