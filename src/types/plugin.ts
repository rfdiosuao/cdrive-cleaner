export type PluginStatus = "active" | "inactive" | "error" | "loading";

export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  status: PluginStatus;
  category: PluginCategory;
  enabled: boolean;
  config: Record<string, unknown>;
  scanCount: number;
  cleanCount: number;
  installedAt: string;
  updatedAt: string;
}

export type PluginCategory = "browser" | "system" | "app" | "development" | "gaming" | "other";

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  category: PluginCategory;
  entryPoint: string;
  permissions: string[];
  minAppVersion: string;
}

export interface PluginScanResult {
  pluginId: string;
  targets: PluginScanTarget[];
  totalSize: number;
  fileCount: number;
}

export interface PluginScanTarget {
  path: string;
  name: string;
  size: number;
  safe: boolean;
  reason: string;
}

export interface PluginCleanResult {
  pluginId: string;
  success: boolean;
  freedSpace: number;
  cleanedFiles: number;
  errors: string[];
}
