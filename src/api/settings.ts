import { invokeCommand } from "./index";
import type { AIConfig, UserPreferences } from "../types/settings";

export async function settingsGet(): Promise<UserPreferences> {
  return invokeCommand<UserPreferences>("settings_get");
}

export async function settingsSet(prefs: UserPreferences): Promise<void> {
  return invokeCommand("settings_set", { prefs });
}

export async function aiConfigGet(): Promise<AIConfig> {
  return invokeCommand<AIConfig>("ai_config_get");
}

export async function aiConfigSet(config: AIConfig): Promise<void> {
  return invokeCommand("ai_config_set", { config });
}

export async function aiTestConnection(): Promise<boolean> {
  return invokeCommand<boolean>("ai_test_connection");
}

export async function systemInfo(): Promise<any> {
  return invokeCommand<any>("system_info");
}
