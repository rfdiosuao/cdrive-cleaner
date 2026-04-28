import { invokeCommand } from "./index";
import type { CleanTask, CleanResult, SafetyScore } from "../types/clean";

export async function cleanPreview(tasks: CleanTask[]): Promise<SafetyScore> {
  return invokeCommand<SafetyScore>("clean_preview", { tasks });
}

export async function cleanExecute(tasks: CleanTask[]): Promise<CleanResult[]> {
  return invokeCommand<CleanResult[]>("clean_execute", { tasks });
}

export async function cleanRestore(restoreId: string): Promise<void> {
  return invokeCommand("clean_restore", { restoreId });
}
