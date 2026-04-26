<script setup lang="ts">
import { ref, onMounted } from "vue";
import { HardDrive, Trash2, Shield, Zap, RefreshCw } from "lucide-vue-next";
import { scanStart, scanProgress, scanResult } from "../api/scan";
import { systemInfo } from "../api/settings";
import { useScanStore } from "../stores/scan";
import type { ScanResult } from "../types/scan";

const scanStore = useScanStore();

const totalSpace = ref("--");
const freeSpace = ref("--");
const usedPercent = ref(0);
const cleanableSpace = ref("--");
const safetyScore = ref("--");
const optimizeCount = ref("--");
const isLoading = ref(false);

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

async function loadSystemInfo() {
  try {
    const info = await systemInfo();
    const cDrive = info.drives.find((d: any) => d.letter === "C:" || d.letter === "C:\\");
    if (cDrive) {
      totalSpace.value = formatBytes(cDrive.total_space);
      freeSpace.value = formatBytes(cDrive.available_space);
      usedPercent.value = Math.round((cDrive.used_space / cDrive.total_space) * 100);
    }
  } catch (e) {
    console.error("获取系统信息失败", e);
  }
}

async function startQuickScan() {
  isLoading.value = true;
  scanStore.setScanning(true);
  scanStore.clearResults();

  try {
    await scanStart("quick");

    const pollProgress = async () => {
      try {
        const progress: any = await scanProgress();
        if (progress) {
          scanStore.setProgress(progress);
        }

        const results: ScanResult[] = await scanResult();
        if (results.length > 0) {
          scanStore.setResults(results);

          const totalCleanable = results.reduce((sum: number, r: ScanResult) => sum + r.totalSize, 0);
          cleanableSpace.value = formatBytes(totalCleanable);

          const avgScore = results.reduce((sum: number, r: ScanResult) => sum + r.safetyScore, 0) / results.length;
          safetyScore.value = avgScore.toFixed(0);

          optimizeCount.value = String(results.length);
        }

        if (progress && progress.phase !== "completed" && progress.phase !== "cancelled" && progress.phase !== "error") {
          setTimeout(pollProgress, 500);
        } else {
          isLoading.value = false;
          scanStore.setScanning(false);
        }
      } catch (e) {
        console.error("轮询进度失败", e);
        isLoading.value = false;
        scanStore.setScanning(false);
      }
    };

    setTimeout(pollProgress, 300);
  } catch (e) {
    console.error("启动扫描失败", e);
    isLoading.value = false;
    scanStore.setScanning(false);
  }
}

onMounted(() => {
  loadSystemInfo();
});
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold text-dark-100">仪表盘</h1>
      <button
        class="btn-primary flex items-center gap-2"
        @click="startQuickScan"
        :disabled="isLoading"
      >
        <RefreshCw :size="16" :class="{ 'animate-spin': isLoading }" />
        {{ isLoading ? "扫描中..." : "快速扫描" }}
      </button>
    </div>

    <div class="grid grid-cols-4 gap-4">
      <div class="card">
        <div class="flex items-center gap-3 mb-3">
          <div class="p-2 rounded-lg bg-accent-blue/15">
            <HardDrive :size="20" class="text-accent-blue" />
          </div>
          <span class="text-sm text-dark-400">C盘总容量</span>
        </div>
        <div class="text-2xl font-bold">{{ totalSpace }}</div>
        <div class="text-xs text-dark-500 mt-1">可用: {{ freeSpace }}</div>
      </div>

      <div class="card">
        <div class="flex items-center gap-3 mb-3">
          <div class="p-2 rounded-lg bg-accent-red/15">
            <Trash2 :size="20" class="text-accent-red" />
          </div>
          <span class="text-sm text-dark-400">可清理空间</span>
        </div>
        <div class="text-2xl font-bold text-accent-red">{{ cleanableSpace }}</div>
        <div class="text-xs text-dark-500 mt-1">{{ isLoading ? '扫描中...' : (scanStore.results.length > 0 ? '扫描完成' : '等待扫描') }}</div>
      </div>

      <div class="card">
        <div class="flex items-center gap-3 mb-3">
          <div class="p-2 rounded-lg bg-accent-green/15">
            <Shield :size="20" class="text-accent-green" />
          </div>
          <span class="text-sm text-dark-400">安全评分</span>
        </div>
        <div class="text-2xl font-bold" :class="safetyScore !== '--' && Number(safetyScore) >= 80 ? 'text-accent-green' : 'text-accent-yellow'">{{ safetyScore }}</div>
        <div class="text-xs text-dark-500 mt-1">{{ safetyScore !== '--' ? '整体安全' : '等待扫描' }}</div>
      </div>

      <div class="card">
        <div class="flex items-center gap-3 mb-3">
          <div class="p-2 rounded-lg bg-accent-purple/15">
            <Zap :size="20" class="text-accent-purple" />
          </div>
          <span class="text-sm text-dark-400">系统优化项</span>
        </div>
        <div class="text-2xl font-bold text-accent-purple">{{ optimizeCount }}</div>
        <div class="text-xs text-dark-500 mt-1">{{ optimizeCount !== '--' ? '项可优化' : '等待扫描' }}</div>
      </div>
    </div>

    <div class="card">
      <h2 class="text-lg font-semibold mb-4">磁盘使用概览</h2>
      <div v-if="usedPercent > 0">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-dark-400">C盘已使用</span>
          <span class="text-sm font-medium">{{ usedPercent }}%</span>
        </div>
        <div class="w-full bg-dark-700 rounded-full h-3">
          <div
            class="h-3 rounded-full transition-all duration-500"
            :class="usedPercent > 90 ? 'bg-accent-red' : usedPercent > 70 ? 'bg-accent-yellow' : 'bg-accent-green'"
            :style="{ width: usedPercent + '%' }"
          ></div>
        </div>
      </div>
      <div v-else class="text-dark-500 text-center py-12">点击「快速扫描」查看C盘详细信息</div>
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
            <span class="text-xs px-2 py-0.5 rounded" :class="{
              'bg-accent-green/15 text-accent-green': result.riskLevel === 'safe',
              'bg-accent-yellow/15 text-accent-yellow': result.riskLevel === 'low',
              'bg-accent-orange/15 text-accent-orange': result.riskLevel === 'medium',
              'bg-accent-red/15 text-accent-red': result.riskLevel === 'high' || result.riskLevel === 'critical',
            }">{{ result.recommendation }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
