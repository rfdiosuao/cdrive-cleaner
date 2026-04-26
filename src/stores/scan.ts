import { defineStore } from "pinia";
import { ref } from "vue";
import type { ScanResult, ScanProgress } from "../types/scan";

export const useScanStore = defineStore("scan", () => {
  const isScanning = ref(false);
  const progress = ref<ScanProgress | null>(null);
  const results = ref<ScanResult[]>([]);

  function setScanning(value: boolean) {
    isScanning.value = value;
  }

  function setProgress(value: ScanProgress) {
    progress.value = value;
  }

  function setResults(value: ScanResult[]) {
    results.value = value;
  }

  function clearResults() {
    results.value = [];
    progress.value = null;
  }

  return {
    isScanning,
    progress,
    results,
    setScanning,
    setProgress,
    setResults,
    clearResults,
  };
});
