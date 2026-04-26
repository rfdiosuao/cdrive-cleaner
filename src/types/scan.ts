export type ScanMode = "quick" | "deep" | "custom";

export interface ScanTarget {
  id: string;
  path: string;
  name: string;
  category: ScanCategory;
  size: number;
  fileCount: number;
  description: string;
}

export type ScanCategory =
  | "temp"
  | "cache"
  | "log"
  | "download"
  | "recycle"
  | "browser"
  | "system"
  | "app"
  | "other";

export interface ScanResult {
  id: string;
  target: ScanTarget;
  files: FileMetadata[];
  totalSize: number;
  fileCount: number;
  safetyScore: number;
  riskLevel: RiskLevel;
  recommendation: string;
  scannedAt: string;
}

export type RiskLevel = "safe" | "low" | "medium" | "high" | "critical";

export interface ScanProgress {
  phase: ScanPhase;
  currentPath: string;
  scannedFiles: number;
  totalFiles: number;
  scannedSize: number;
  elapsedTimeMs: number;
  estimatedTimeRemainingMs: number;
  percent: number;
}

export type ScanPhase = "initializing" | "scanning" | "analyzing" | "evaluating" | "completed" | "cancelled" | "error";

export interface FileMetadata {
  path: string;
  name: string;
  size: number;
  modifiedAt: string;
  createdAt: string;
  isDirectory: boolean;
  extension: string;
  category: ScanCategory;
  hash: string;
}
