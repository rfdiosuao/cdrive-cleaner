<script setup lang="ts">
import { computed, ref } from "vue";
import { CheckCircle2, Circle, RotateCcw, ScanSearch, Square, Trash2 } from "lucide-vue-next";
import { cleanExecute, cleanPreview, cleanProgress, cleanStop } from "../api/clean";
import { scanProgress, scanResult, scanStart, scanStop } from "../api/scan";
import { useScanStore } from "../stores/scan";
import type { CleanPhase, CleanPriority, CleanProgress, CleanTask } from "../types/clean";
import type { FileMetadata, ScanProgress, ScanResult } from "../types/scan";

const scanStore = useScanStore();
const selectedIds = ref<Set<string>>(new Set());
const isScanning = ref(false);
const isCleaning = ref(false);
const cleanStatus = ref<string | null>(null);
const scanStatus = ref("尚未扫描");
const cleanFinishedAt = ref("");
const scanFinishedAt = ref("");
const filterCategory = ref<string>("all");
const currentCleanProgress = ref<CleanProgress | null>(null);

const terminalScanPhases = ["completed", "cancelled", "error"];
const terminalCleanPhases: CleanPhase[] = ["completed", "failed", "cancelled"];

const categoryNames: Record<string, string> = {
  all: "全部",
  temp: "临时文件",
  cache: "缓存文件",
  log: "日志文件",
  download: "下载文件",
  recycle: "回收站",
  browser: "浏览器缓存",
  system: "系统文件",
  app: "应用数据",
  other: "其他",
};

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

function scanPhaseLabel(phase?: ScanProgress["phase"]) {
  const labels: Record<string, string> = {
    initializing: "正在初始化",
    scanning: "正在扫描文件",
    analyzing: "正在分析结果",
    evaluating: "正在评估风险",
    completed: "扫描完成",
    cancelled: "扫描已取消",
    error: "扫描出错",
  };

  return phase ? labels[phase] ?? "扫描中" : "尚未扫描";
}

function cleanPhaseLabel(phase?: CleanPhase) {
  const labels: Record<CleanPhase, string> = {
    preparing: "正在准备清理",
    backing_up: "正在备份",
    cleaning: "正在清理文件",
    verifying: "正在验证结果",
    completed: "清理完成",
    failed: "清理失败",
    cancelled: "清理已取消",
  };

  return phase ? labels[phase] : "等待清理";
}

const filteredResults = computed(() => {
  if (filterCategory.value === "all") return scanStore.results;
  return scanStore.results.filter((result) => result.target.category === filterCategory.value);
});

const totalSelectedSize = computed(() => {
  return scanStore.results
    .filter((result) => selectedIds.value.has(result.id))
    .reduce((sum, result) => sum + result.totalSize, 0);
});

const totalResultSize = computed(() => {
  return scanStore.results.reduce((sum, result) => sum + result.totalSize, 0);
});

const scanProgressPercent = computed(() => {
  const percent = scanStore.progress?.percent ?? 0;
  if (isScanning.value && percent <= 0) return 2;
  return Math.min(100, Math.max(0, percent));
});

const cleanProgressPercent = computed(() => {
  const percent = currentCleanProgress.value?.percent ?? 0;
  if (isCleaning.value && percent <= 0) return 2;
  return Math.min(100, Math.max(0, percent));
});

const scanProgressText = computed(() => {
  const progress = scanStore.progress;
  if (!progress) return "点击扫描后开始统计进度。";
  if (progress.phase === "completed") {
    return `完成：发现 ${scanStore.results.length} 类项目，可释放 ${formatBytes(totalResultSize.value)}。`;
  }
  if (progress.phase === "cancelled") return "扫描已取消，当前结果可能不完整。";
  if (progress.phase === "error") return "扫描失败，请重试。";

  return `已扫描 ${progress.scannedFiles.toLocaleString()} 个文件，累计 ${formatBytes(progress.scannedSize || 0)}。`;
});

const cleanProgressText = computed(() => {
  const progress = currentCleanProgress.value;
  if (!progress) return "选择清理项后，这里会显示备份、清理和验证进度。";
  if (progress.phase === "completed") {
    return `清理完成：释放 ${formatBytes(progress.freedSpace)}，处理 ${progress.completedFiles}/${progress.totalFiles} 个任务。`;
  }
  if (progress.phase === "cancelled") return "清理已取消，部分任务可能已经完成。";
  if (progress.phase === "failed") return "清理失败，请查看下方提示。";

  return `${cleanPhaseLabel(progress.phase)}：${progress.completedFiles}/${progress.totalFiles} 个任务，已释放 ${formatBytes(progress.freedSpace)}。`;
});

function toggleSelect(id: string) {
  if (isCleaning.value) return;

  if (selectedIds.value.has(id)) {
    selectedIds.value.delete(id);
  } else {
    selectedIds.value.add(id);
  }
}

function selectAllVisible() {
  const visibleIds = filteredResults.value.map((result) => result.id);
  const allVisibleSelected = visibleIds.every((id) => selectedIds.value.has(id));

  if (allVisibleSelected) {
    visibleIds.forEach((id) => selectedIds.value.delete(id));
    return;
  }

  visibleIds.forEach((id) => selectedIds.value.add(id));
}

async function startScan(mode: "quick" | "deep" = "deep") {
  isScanning.value = true;
  cleanStatus.value = null;
  scanFinishedAt.value = "";
  scanStatus.value = "正在启动扫描";
  selectedIds.value.clear();
  scanStore.setScanning(true);
  scanStore.clearResults();

  try {
    await scanStart(mode);

    const poll = async () => {
      try {
        const progress = await scanProgress();
        scanStore.setProgress(progress);
        scanStatus.value = scanPhaseLabel(progress.phase);

        const results = await scanResult();
        if (results.length > 0) {
          scanStore.setResults(results);
        }

        if (!terminalScanPhases.includes(progress.phase)) {
          window.setTimeout(poll, 500);
          return;
        }

        const finalResults = await scanResult();
        scanStore.setResults(finalResults);
        isScanning.value = false;
        scanStore.setScanning(false);
        scanStatus.value = scanPhaseLabel(progress.phase);
        scanFinishedAt.value = new Date().toLocaleTimeString("zh-CN", { hour12: false });
      } catch {
        isScanning.value = false;
        scanStore.setScanning(false);
        scanStatus.value = "扫描出错";
      }
    };

    window.setTimeout(poll, 300);
  } catch {
    isScanning.value = false;
    scanStore.setScanning(false);
    scanStatus.value = "启动失败";
  }
}

async function stopScan() {
  if (!isScanning.value) return;

  try {
    await scanStop();
    scanStatus.value = "正在取消";
  } catch {
    scanStatus.value = "取消失败";
  }
}

async function stopClean() {
  if (!isCleaning.value) return;

  try {
    await cleanStop();
    cleanStatus.value = "正在取消清理";
  } catch {
    cleanStatus.value = "取消清理失败";
  }
}

function priorityForRisk(riskLevel: ScanResult["riskLevel"]): CleanPriority {
  if (riskLevel === "critical") return "critical";
  if (riskLevel === "high") return "high";
  if (riskLevel === "medium") return "normal";
  return "low";
}

function buildCleanTasks(result: ScanResult): CleanTask[] {
  const files = result.files.length > 0
    ? result.files
    : [{
        path: result.target.path,
        name: result.target.name,
        size: result.totalSize,
        isDirectory: false,
      } as FileMetadata];

  return files.map((file, index) => ({
    id: `${result.id}-${index}`,
    targetId: result.id,
    targetPath: file.path,
    targetName: file.name || result.target.name,
    action: "delete",
    priority: priorityForRisk(result.riskLevel),
    riskLevel: result.riskLevel,
    size: file.size,
    fileCount: 1,
    backupRequired: true,
  }));
}

function startCleanPolling() {
  const poll = async () => {
    try {
      const progress = await cleanProgress();
      if (progress) {
        currentCleanProgress.value = progress;
      }

      if (progress && !terminalCleanPhases.includes(progress.phase)) {
        window.setTimeout(poll, 300);
      }
    } catch {
      if (isCleaning.value) {
        window.setTimeout(poll, 600);
      }
    }
  };

  window.setTimeout(poll, 150);
}

async function cleanSelected() {
  const selectedResults = scanStore.results.filter((result) => selectedIds.value.has(result.id));
  if (selectedResults.length === 0 || isCleaning.value) return;

  const tasks = selectedResults.flatMap(buildCleanTasks);
  if (tasks.length === 0) return;

  isCleaning.value = true;
  cleanFinishedAt.value = "";
  cleanStatus.value = "正在生成清理预览";
  currentCleanProgress.value = {
    taskId: "",
    currentFile: "",
    completedFiles: 0,
    totalFiles: tasks.length,
    freedSpace: 0,
    percent: 0,
    phase: "preparing",
  };

  try {
    const preview = await cleanPreview(tasks);
    if (preview.warnings.length > 0) {
      const confirmed = window.confirm(`发现 ${preview.warnings.length} 条风险提示，是否继续清理？`);
      if (!confirmed) {
        cleanStatus.value = "已取消清理";
        currentCleanProgress.value = {
          ...currentCleanProgress.value,
          phase: "cancelled",
        };
        return;
      }
    }

    cleanStatus.value = "正在清理";
    startCleanPolling();
    const results = await cleanExecute(tasks);
    const finalProgress = await cleanProgress();
    if (finalProgress) {
      currentCleanProgress.value = finalProgress;
    }

    const failed = results.filter((result) => !result.success).length;
    const cleaned = results.reduce((sum, result) => sum + result.cleanedFiles, 0);
    const freed = results.reduce((sum, result) => sum + result.freedSpace, 0);

    if (failed === 0) {
      const selected = new Set(selectedIds.value);
      scanStore.setResults(scanStore.results.filter((result) => !selected.has(result.id)));
      selectedIds.value.clear();
    }

    cleanFinishedAt.value = new Date().toLocaleTimeString("zh-CN", { hour12: false });
    cleanStatus.value = failed === 0
      ? `已清理 ${cleaned} 个文件，释放 ${formatBytes(freed)}。`
      : `清理完成，${failed} 个任务失败，已清理 ${cleaned} 个文件。`;
  } catch (error: any) {
    cleanStatus.value = error?.message || "清理失败";
    currentCleanProgress.value = currentCleanProgress.value
      ? { ...currentCleanProgress.value, phase: "failed" }
      : null;
  } finally {
    isCleaning.value = false;
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-dark-100">扫描结果</h1>
        <p class="mt-1 text-sm text-dark-500">扫描和清理都会显示明确的阶段、进度和完成时间。</p>
      </div>
      <div class="flex items-center gap-3">
        <button class="btn-secondary flex items-center gap-2" @click="selectAllVisible" :disabled="filteredResults.length === 0 || isCleaning">
          <CheckCircle2 :size="16" />
          选择当前列表
        </button>
        <button class="btn-secondary flex items-center gap-2" @click="startScan('quick')" :disabled="isScanning || isCleaning">
          <RotateCcw :size="16" :class="{ 'animate-spin': isScanning }" />
          快速扫描
        </button>
        <button class="btn-primary flex items-center gap-2" @click="startScan('deep')" :disabled="isScanning || isCleaning">
          <ScanSearch :size="16" :class="{ 'animate-spin': isScanning }" />
          {{ isScanning ? "扫描中" : "深度扫描" }}
        </button>
        <button v-if="isScanning" class="btn-danger flex items-center gap-2" @click="stopScan">
          <Square :size="14" />
          停止
        </button>
      </div>
    </div>

    <div class="status-panel">
      <div class="flex items-start justify-between gap-4">
        <div>
          <div class="flex items-center gap-2">
            <CheckCircle2 v-if="scanStore.progress?.phase === 'completed'" :size="18" class="text-accent-green" />
            <ScanSearch v-else :size="18" :class="isScanning ? 'animate-pulse text-accent-blue' : 'text-dark-500'" />
            <span class="font-semibold">{{ scanStatus }}</span>
          </div>
          <p class="mt-2 text-sm text-dark-500">{{ scanProgressText }}</p>
          <p v-if="scanStore.progress?.currentPath && isScanning" class="mt-1 max-w-3xl truncate text-xs text-dark-500">
            当前路径：{{ scanStore.progress.currentPath }}
          </p>
        </div>
        <div class="text-right text-sm text-dark-500">
          <div class="text-lg font-bold text-dark-100">{{ scanProgressPercent.toFixed(0) }}%</div>
          <div v-if="scanFinishedAt">完成于 {{ scanFinishedAt }}</div>
        </div>
      </div>
      <div class="status-strip mt-4">
        <div class="status-strip-fill" :style="{ width: `${scanProgressPercent}%` }"></div>
      </div>
    </div>

    <div v-if="currentCleanProgress || isCleaning || cleanStatus" class="status-panel">
      <div class="flex items-start justify-between gap-4">
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <CheckCircle2 v-if="currentCleanProgress?.phase === 'completed'" :size="18" class="text-accent-green" />
            <Trash2 v-else :size="18" :class="isCleaning ? 'animate-pulse text-accent-red' : 'text-dark-500'" />
            <span class="font-semibold">{{ cleanPhaseLabel(currentCleanProgress?.phase) }}</span>
          </div>
          <p class="mt-2 text-sm text-dark-500">{{ cleanProgressText }}</p>
          <p v-if="currentCleanProgress?.currentFile && isCleaning" class="mt-1 max-w-3xl truncate text-xs text-dark-500">
            当前文件：{{ currentCleanProgress.currentFile }}
          </p>
          <p v-if="cleanStatus" class="mt-2 text-sm text-dark-300">{{ cleanStatus }}</p>
        </div>
        <div class="shrink-0 text-right text-sm text-dark-500">
          <div class="text-lg font-bold text-dark-100">{{ cleanProgressPercent.toFixed(0) }}%</div>
          <div v-if="cleanFinishedAt">完成于 {{ cleanFinishedAt }}</div>
          <button v-if="isCleaning" class="btn-danger mt-2 inline-flex items-center gap-2 px-3 py-1.5 text-sm" @click="stopClean">
            <Square :size="14" />
            停止清理
          </button>
        </div>
      </div>
      <div class="status-strip mt-4">
        <div class="status-strip-fill" :style="{ width: `${cleanProgressPercent}%` }"></div>
      </div>
    </div>

    <div v-if="scanStore.results.length > 0" class="flex flex-wrap items-center gap-2">
      <button
        v-for="(name, key) in categoryNames"
        :key="key"
        class="px-3 py-1 text-xs rounded-full transition-colors"
        :class="filterCategory === key ? 'bg-accent-blue text-white' : 'bg-dark-700 text-dark-400 hover:bg-dark-600'"
        @click="filterCategory = key"
      >
        {{ name }}
      </button>
      <div class="ml-auto text-sm text-dark-500">
        共 {{ scanStore.results.length }} 类，{{ formatBytes(totalResultSize) }}
      </div>
    </div>

    <div v-if="scanStore.results.length > 0" class="space-y-2">
      <div
        v-for="result in filteredResults"
        :key="result.id"
        class="card flex items-center gap-4 cursor-pointer hover:border-accent-blue/30 transition-colors"
        :class="{ 'opacity-60': isCleaning }"
        @click="toggleSelect(result.id)"
      >
        <component
          :is="selectedIds.has(result.id) ? CheckCircle2 : Circle"
          :size="20"
          :class="selectedIds.has(result.id) ? 'text-accent-blue' : 'text-dark-600'"
        />
        <div class="min-w-0 flex-1">
          <div class="flex items-center justify-between gap-4">
            <span class="font-medium">{{ result.target.name }}</span>
            <span class="text-lg font-bold">{{ formatBytes(result.totalSize) }}</span>
          </div>
          <div class="mt-1 flex items-center justify-between gap-4">
            <span class="truncate text-xs text-dark-500">{{ result.target.path }} · {{ result.fileCount }} 个文件</span>
            <span
              class="shrink-0 text-xs px-2 py-0.5 rounded"
              :class="{
                'bg-accent-green/15 text-accent-green': result.riskLevel === 'safe',
                'bg-accent-yellow/15 text-accent-yellow': result.riskLevel === 'low',
                'bg-accent-orange/15 text-accent-orange': result.riskLevel === 'medium',
                'bg-accent-red/15 text-accent-red': result.riskLevel === 'high' || result.riskLevel === 'critical',
              }"
            >
              {{ result.recommendation }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="card">
      <div class="empty-panel text-dark-500">
        <ScanSearch :size="48" class="mx-auto mb-4 text-dark-600" />
        <p>尚未进行扫描</p>
        <p class="text-sm mt-2">点击上方按钮开始扫描 C 盘，完成后会显示结果列表。</p>
      </div>
    </div>

    <div
      v-if="selectedIds.size > 0"
      class="fixed bottom-6 left-1/2 -translate-x-1/2 bg-dark-800 border border-dark-600 rounded-xl px-6 py-3 flex items-center gap-4 shadow-xl"
    >
      <span class="text-sm text-dark-400">已选择 <span class="text-accent-blue font-bold">{{ selectedIds.size }}</span> 项</span>
      <span class="text-sm text-dark-400">共 <span class="text-accent-red font-bold">{{ formatBytes(totalSelectedSize) }}</span></span>
      <button class="btn-primary flex items-center gap-2" @click="cleanSelected" :disabled="isCleaning">
        <Trash2 :size="16" :class="{ 'animate-pulse': isCleaning }" />
        {{ isCleaning ? "清理中" : "清理选中项" }}
      </button>
    </div>
  </div>
</template>
