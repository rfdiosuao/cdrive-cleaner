<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { CheckCircle2, HardDrive, RefreshCw, Shield, Trash2, Zap } from "lucide-vue-next";
import { scanProgress, scanResult, scanStart } from "../api/scan";
import { systemInfo } from "../api/settings";
import { useScanStore } from "../stores/scan";
import type { ScanProgress, ScanResult } from "../types/scan";

const scanStore = useScanStore();

const totalSpace = ref("--");
const freeSpace = ref("--");
const usedPercent = ref(0);
const cleanableSpace = ref("--");
const safetyScore = ref("--");
const optimizeCount = ref("--");
const isLoading = ref(false);
const scanStatus = ref("等待扫描");
const finishedAt = ref("");

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

function phaseLabel(phase?: ScanProgress["phase"]) {
  const labels: Record<string, string> = {
    initializing: "正在初始化",
    scanning: "正在扫描文件",
    analyzing: "正在分析结果",
    evaluating: "正在评估风险",
    completed: "扫描完成",
    cancelled: "扫描已取消",
    error: "扫描出错",
  };

  return phase ? labels[phase] ?? "扫描中" : "等待扫描";
}

const progressPercent = computed(() => {
  const percent = scanStore.progress?.percent ?? 0;
  if (isLoading.value && percent <= 0) return 2;
  return Math.min(100, Math.max(0, percent));
});

const scanSummary = computed(() => {
  const progress = scanStore.progress;
  if (!progress) return "点击快速扫描后，这里会显示实时进度。";
  if (progress.phase === "completed") {
    return `扫描完成：共发现 ${scanStore.results.length} 类可处理项目，可释放 ${cleanableSpace.value}。`;
  }
  if (progress.phase === "error") return "扫描过程中出现错误，请稍后重试。";
  if (progress.phase === "cancelled") return "扫描已取消，结果可能不完整。";

  const scannedSize = formatBytes(progress.scannedSize || 0);
  return `已扫描 ${progress.scannedFiles.toLocaleString()} 个文件，累计 ${scannedSize}`;
});

function updateResultStats(results: ScanResult[]) {
  if (results.length === 0) {
    cleanableSpace.value = "--";
    safetyScore.value = "--";
    optimizeCount.value = "--";
    return;
  }

  const totalCleanable = results.reduce((sum, result) => sum + result.totalSize, 0);
  const avgScore = results.reduce((sum, result) => sum + result.safetyScore, 0) / results.length;

  cleanableSpace.value = formatBytes(totalCleanable);
  safetyScore.value = avgScore.toFixed(0);
  optimizeCount.value = String(results.length);
}

async function loadSystemInfo() {
  try {
    const info = await systemInfo();
    const cDrive = info.drives.find((drive: any) => drive.letter === "C:" || drive.letter === "C:\\");
    if (cDrive) {
      totalSpace.value = formatBytes(cDrive.totalSize);
      freeSpace.value = formatBytes(cDrive.availableSpace);
      usedPercent.value = cDrive.totalSize > 0
        ? Math.round((cDrive.usedSpace / cDrive.totalSize) * 100)
        : 0;
    }
  } catch (error) {
    console.error("获取系统信息失败", error);
  }
}

async function startQuickScan() {
  isLoading.value = true;
  scanStatus.value = "正在启动扫描";
  finishedAt.value = "";
  scanStore.setScanning(true);
  scanStore.clearResults();
  updateResultStats([]);

  try {
    await scanStart("quick");

    const pollProgress = async () => {
      try {
        const progress = await scanProgress();
        scanStore.setProgress(progress);
        scanStatus.value = phaseLabel(progress.phase);

        const results = await scanResult();
        if (results.length > 0) {
          scanStore.setResults(results);
          updateResultStats(results);
        }

        if (!["completed", "cancelled", "error"].includes(progress.phase)) {
          window.setTimeout(pollProgress, 500);
          return;
        }

        const finalResults = await scanResult();
        scanStore.setResults(finalResults);
        updateResultStats(finalResults);
        isLoading.value = false;
        scanStore.setScanning(false);
        scanStatus.value = phaseLabel(progress.phase);
        finishedAt.value = new Date().toLocaleTimeString("zh-CN", { hour12: false });
      } catch (error) {
        console.error("轮询扫描进度失败", error);
        isLoading.value = false;
        scanStore.setScanning(false);
        scanStatus.value = "扫描出错";
      }
    };

    window.setTimeout(pollProgress, 300);
  } catch (error) {
    console.error("启动扫描失败", error);
    isLoading.value = false;
    scanStore.setScanning(false);
    scanStatus.value = "启动失败";
  }
}

onMounted(() => {
  loadSystemInfo();
  updateResultStats(scanStore.results);
  if (scanStore.progress) {
    scanStatus.value = phaseLabel(scanStore.progress.phase);
  }
});
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-dark-100">仪表盘</h1>
        <p class="mt-1 text-sm text-dark-500">查看 C 盘空间、扫描进度和清理建议。</p>
      </div>
      <button
        class="btn-primary flex items-center gap-2"
        :disabled="isLoading"
        @click="startQuickScan"
      >
        <RefreshCw :size="16" :class="{ 'animate-spin': isLoading }" />
        {{ isLoading ? "扫描中" : "快速扫描" }}
      </button>
    </div>

    <div class="status-panel">
      <div class="flex items-start justify-between gap-4">
        <div>
          <div class="flex items-center gap-2">
            <CheckCircle2
              v-if="scanStore.progress?.phase === 'completed'"
              :size="18"
              class="text-accent-green"
            />
            <RefreshCw
              v-else
              :size="18"
              :class="isLoading ? 'animate-spin text-accent-blue' : 'text-dark-500'"
            />
            <span class="font-semibold">{{ scanStatus }}</span>
          </div>
          <p class="mt-2 text-sm text-dark-500">{{ scanSummary }}</p>
        </div>
        <div class="text-right text-sm text-dark-500">
          <div class="text-lg font-bold text-dark-100">{{ progressPercent.toFixed(0) }}%</div>
          <div v-if="finishedAt">完成于 {{ finishedAt }}</div>
        </div>
      </div>
      <div class="status-strip mt-4">
        <div class="status-strip-fill" :style="{ width: `${progressPercent}%` }"></div>
      </div>
    </div>

    <div class="grid grid-cols-4 gap-4">
      <div class="card">
        <div class="flex items-center gap-3 mb-3">
          <div class="p-2 rounded-lg bg-accent-blue/15">
            <HardDrive :size="20" class="text-accent-blue" />
          </div>
          <span class="text-sm text-dark-400">C 盘总容量</span>
        </div>
        <div class="text-2xl font-bold">{{ totalSpace }}</div>
        <div class="text-xs text-dark-500 mt-1">可用：{{ freeSpace }}</div>
      </div>

      <div class="card">
        <div class="flex items-center gap-3 mb-3">
          <div class="p-2 rounded-lg bg-accent-red/15">
            <Trash2 :size="20" class="text-accent-red" />
          </div>
          <span class="text-sm text-dark-400">可清理空间</span>
        </div>
        <div class="text-2xl font-bold text-accent-red">{{ cleanableSpace }}</div>
        <div class="text-xs text-dark-500 mt-1">{{ scanStore.results.length > 0 ? "来自最近一次扫描" : "等待扫描" }}</div>
      </div>

      <div class="card">
        <div class="flex items-center gap-3 mb-3">
          <div class="p-2 rounded-lg bg-accent-green/15">
            <Shield :size="20" class="text-accent-green" />
          </div>
          <span class="text-sm text-dark-400">安全评分</span>
        </div>
        <div
          class="text-2xl font-bold"
          :class="safetyScore !== '--' && Number(safetyScore) >= 80 ? 'text-accent-green' : 'text-accent-yellow'"
        >
          {{ safetyScore }}
        </div>
        <div class="text-xs text-dark-500 mt-1">{{ safetyScore !== "--" ? "分数越高越安全" : "等待扫描" }}</div>
      </div>

      <div class="card">
        <div class="flex items-center gap-3 mb-3">
          <div class="p-2 rounded-lg bg-accent-purple/15">
            <Zap :size="20" class="text-accent-purple" />
          </div>
          <span class="text-sm text-dark-400">优化项目</span>
        </div>
        <div class="text-2xl font-bold text-accent-purple">{{ optimizeCount }}</div>
        <div class="text-xs text-dark-500 mt-1">{{ optimizeCount !== "--" ? "项可处理" : "等待扫描" }}</div>
      </div>
    </div>

    <div class="card">
      <h2 class="text-lg font-semibold mb-4">磁盘使用概览</h2>
      <div v-if="usedPercent > 0">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-dark-400">C 盘已使用</span>
          <span class="text-sm font-medium">{{ usedPercent }}%</span>
        </div>
        <div class="w-full bg-dark-700 rounded-full h-3">
          <div
            class="h-3 rounded-full transition-all duration-500"
            :class="usedPercent > 90 ? 'bg-accent-red' : usedPercent > 70 ? 'bg-accent-yellow' : 'bg-accent-green'"
            :style="{ width: `${usedPercent}%` }"
          ></div>
        </div>
      </div>
      <div v-else class="empty-panel text-dark-500">正在读取 C 盘空间信息。</div>
    </div>

    <div v-if="scanStore.results.length > 0" class="card">
      <h2 class="text-lg font-semibold mb-4">清理建议</h2>
      <div class="space-y-3">
        <div
          v-for="result in scanStore.results.slice(0, 5)"
          :key="result.id"
          class="flex items-center justify-between p-3 rounded-lg bg-dark-800/50"
        >
          <div class="flex items-center gap-3">
            <div
              class="w-2 h-2 rounded-full"
              :class="{
                'bg-accent-green': result.riskLevel === 'safe',
                'bg-accent-yellow': result.riskLevel === 'low',
                'bg-accent-orange': result.riskLevel === 'medium',
                'bg-accent-red': result.riskLevel === 'high' || result.riskLevel === 'critical',
              }"
            ></div>
            <span class="text-sm">{{ result.target.name }}</span>
          </div>
          <div class="flex items-center gap-4">
            <span class="text-sm text-dark-400">{{ formatBytes(result.totalSize) }}</span>
            <span
              class="text-xs px-2 py-0.5 rounded"
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
  </div>
</template>
