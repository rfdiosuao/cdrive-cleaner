export type CloudModelProvider = "openai" | "anthropic" | "deepseek" | "qwen" | "custom";

export interface AIConfig {
  provider: "local" | CloudModelProvider;
  apiKey: string;
  model: string;
  baseUrl: string;
  temperature: number;
  maxTokens: number;
}

export interface UserPreferences {
  autoScanOnStart: boolean;
  scanMode: "quick" | "deep" | "custom";
  cleanConfirmRequired: boolean;
  backupBeforeClean: boolean;
  language: string;
  theme: "dark" | "light" | "system";
}

export interface WhitelistConfig {
  paths: WhitelistEntry[];
  globalEnabled: boolean;
}

export interface WhitelistEntry {
  id: string;
  path: string;
  reason: string;
  addedAt: string;
  enabled: boolean;
}

export interface AppConfig {
  ai: AIConfig;
  preferences: UserPreferences;
  whitelist: WhitelistConfig;
}
