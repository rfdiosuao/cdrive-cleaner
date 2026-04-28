import { invokeCommand } from "./index";
import type { ScanResult, ScanProgress } from "../types/scan";

export async function scanStart(mode: string): Promise<void> {
  return invokeCommand("scan_start", { mode });
}

export async function scanStop(): Promise<void> {
  return invokeCommand("scan_stop");
}

export async function scanProgress(): Promise<ScanProgress> {
  return invokeCommand<ScanProgress>("scan_progress");
}

export async function scanResult(): Promise<ScanResult[]> {
  return invokeCommand<ScanResult[]>("scan_result");
}
