export type CleanPriority = "low" | "normal" | "high" | "critical";
export type RiskLevel = "safe" | "low" | "medium" | "high" | "critical";

export interface CleanTask {
  id: string;
  targetId: string;
  targetPath: string;
  targetName: string;
  action: CleanAction;
  priority: CleanPriority;
  riskLevel: RiskLevel;
  size: number;
  fileCount: number;
  backupRequired: boolean;
}

export type CleanAction = "delete" | "move" | "compress" | "archive";

export interface CleanResult {
  taskId: string;
  success: boolean;
  freedSpace: number;
  cleanedFiles: number;
  failedFiles: number;
  backupPath: string;
  elapsedTime: number;
  errors: CleanError[];
}

export interface CleanError {
  filePath: string;
  errorCode: string;
  message: string;
}

export interface SafetyScore {
  overall: number;
  categories: SafetyCategory[];
  warnings: string[];
  recommendations: string[];
}

export interface SafetyCategory {
  name: string;
  score: number;
  level: RiskLevel;
  description: string;
}

export interface CleanProgress {
  taskId: string;
  currentFile: string;
  completedFiles: number;
  totalFiles: number;
  freedSpace: number;
  percent: number;
  phase: CleanPhase;
}

export type CleanPhase = "preparing" | "backing_up" | "cleaning" | "verifying" | "completed" | "failed" | "cancelled";

export interface RestorePoint {
  id: string;
  createdAt: string;
  tasks: CleanTask[];
  totalSize: number;
  description: string;
}
