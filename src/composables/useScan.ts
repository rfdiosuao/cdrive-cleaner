import { ref } from "vue";
import { useScanStore } from "../stores/scan";
import { scanStart, scanStop, scanProgress, scanResult } from "../api/scan";
import type { ScanMode, ScanProgress as ScanProgressType, ScanResult as ScanResultType } from "../types/scan";

export function useScan() {
  const store = useScanStore();
  const error = ref<string | null>(null);

  async function startScan(mode: ScanMode = "quick") {
    try {
      error.value = null;
      store.setScanning(true);
      store.clearResults();
      await scanStart(mode);
    } catch (e: any) {
      error.value = e?.message || "扫描启动失败";
      store.setScanning(false);
    }
  }

  async function stopScan() {
    try {
      await scanStop();
      store.setScanning(false);
    } catch (e: any) {
      error.value = e?.message || "停止扫描失败";
    }
  }

  async function fetchProgress(): Promise<ScanProgressType | null> {
    try {
      const p = await scanProgress();
      store.setProgress(p);
      return p;
    } catch (e: any) {
      error.value = e?.message || "获取扫描进度失败";
      return null;
    }
  }

  async function fetchResults(): Promise<ScanResultType[]> {
    try {
      const r = await scanResult();
      store.setResults(r);
      store.setScanning(false);
      return r;
    } catch (e: any) {
      error.value = e?.message || "获取扫描结果失败";
      return [];
    }
  }

  return {
    isScanning: store.isScanning,
    progress: store.progress,
    results: store.results,
    error,
    startScan,
    stopScan,
    fetchProgress,
    fetchResults,
  };
}
