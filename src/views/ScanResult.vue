<script setup lang="ts">
import { ref, computed } from "vue";
import { ScanSearch, CheckCircle2, Circle, Trash2 } from "lucide-vue-next";
import { scanStart, scanProgress, scanResult } from "../api/scan";
import { useScanStore } from "../stores/scan";
import type { ScanResult } from "../types/scan";

const scanStore = useScanStore();
const selectedIds = ref<Set<string>>(new Set());
const isScanning = ref(false);
const filterCategory = ref<string>("all");

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

const filteredResults = computed(() => {
  if (filterCategory.value === "all") return scanStore.results;
  return scanStore.results.filter((r) => r.target.category === filterCategory.value);
});

const totalSelectedSize = computed(() => {
  return scanStore.results
    .filter((r) => selectedIds.value.has(r.id))
    .reduce((sum, r) => sum + r.totalSize, 0);
});

function toggleSelect(id: string) {
  if (selectedIds.value.has(id)) {
    selectedIds.value.delete(id);
  } else {
    selectedIds.value.add(id);
  }
}

function selectAll() {
  if (selectedIds.value.size === scanStore.results.length) {
    selectedIds.value.clear();
  } else {
    scanStore.results.forEach((r) => selectedIds.value.add(r.id));
  }
}

async function startScan() {
  isScanning.value = true;
  scanStore.setScanning(true);
  scanStore.clearResults();

  try {
    await scanStart("deep");

    const poll = async () => {
      try {
        const progress: any = await scanProgress();
        if (progress) scanStore.setProgress(progress);

        const results: ScanResult[] = await scanResult();
        if (results.length > 0) scanStore.setResults(results);

        if (progress && progress.phase !== "completed" && progress.phase !== "cancelled" && progress.phase !== "error") {
          setTimeout(poll, 500);
        } else {
          isScanning.value = false;
          scanStore.setScanning(false);
        }
      } catch {
        isScanning.value = false;
        scanStore.setScanning(false);
      }
    };

    setTimeout(poll, 300);
  } catch {
    isScanning.value = false;
    scanStore.setScanning(false);
  }
}

const categoryNames: Record<string, string> = {
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
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold text-dark-100">扫描结果</h1>
      <div class="flex items-center gap-3">
        <button class="btn-secondary flex items-center gap-2" @click="selectAll">
          <CheckCircle2 :size="16" />
          {{ selectedIds.size === scanStore.results.length ? '取消全选' : '全选' }}
        </button>
        <button class="btn-primary flex items-center gap-2" @click="startScan" :disabled="isScanning">
          <ScanSearch :size="16" :class="{ 'animate-spin': isScanning }" />
          {{ isScanning ? '扫描中...' : '深度扫描' }}
        </button>
      </div>
    </div>

    <div v-if="isScanning && scanStore.progress" class="card">
      <div class="flex items-center justify-between mb-2">
        <span class="text-sm text-dark-400">扫描进度</span>
        <span class="text-sm font-medium">{{ scanStore.progress.percent.toFixed(1) }}%</span>
      </div>
      <div class="w-full bg-dark-700 rounded-full h-2">
        <div class="h-2 rounded-full bg-accent-blue transition-all duration-300" :style="{ width: scanStore.progress.percent + '%' }"></div>
      </div>
      <div class="flex justify-between mt-2 text-xs text-dark-500">
        <span>{{ scanStore.progress.currentPath }}</span>
        <span>{{ scanStore.progress.scannedFiles }} 文件已扫描</span>
      </div>
    </div>

    <div v-if="scanStore.results.length > 0" class="flex items-center gap-2">
      <button
        v-for="(name, key) in categoryNames"
        :key="key"
        class="px-3 py-1 text-xs rounded-full transition-colors"
        :class="filterCategory === key ? 'bg-accent-blue text-white' : 'bg-dark-700 text-dark-400 hover:bg-dark-600'"
        @click="filterCategory = key"
      >
        {{ name }}
      </button>
      <button
        class="px-3 py-1 text-xs rounded-full transition-colors"
        :class="filterCategory === 'all' ? 'bg-accent-blue text-white' : 'bg-dark-700 text-dark-400 hover:bg-dark-600'"
        @click="filterCategory = 'all'"
      >
        全部
      </button>
    </div>

    <div v-if="scanStore.results.length > 0" class="space-y-2">
      <div
        v-for="result in filteredResults"
        :key="result.id"
        class="card flex items-center gap-4 cursor-pointer hover:border-accent-blue/30 transition-colors"
        @click="toggleSelect(result.id)"
      >
        <component
          :is="selectedIds.has(result.id) ? CheckCircle2 : Circle"
          :size="20"
          :class="selectedIds.has(result.id) ? 'text-accent-blue' : 'text-dark-600'"
        />
        <div class="flex-1">
          <div class="flex items-center justify-between">
            <span class="font-medium">{{ result.target.name }}</span>
            <span class="text-lg font-bold">{{ formatBytes(result.totalSize) }}</span>
          </div>
          <div class="flex items-center justify-between mt-1">
            <span class="text-xs text-dark-500">{{ result.target.path }} · {{ result.fileCount }} 个文件</span>
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

    <div v-else class="card">
      <div class="text-dark-500 text-center py-12">
        <ScanSearch :size="48" class="mx-auto mb-4 text-dark-600" />
        <p>尚未进行扫描</p>
        <p class="text-sm mt-2">点击上方按钮开始扫描C盘</p>
      </div>
    </div>

    <div v-if="selectedIds.size > 0" class="fixed bottom-6 left-1/2 -translate-x-1/2 bg-dark-800 border border-dark-600 rounded-xl px-6 py-3 flex items-center gap-4 shadow-xl">
      <span class="text-sm text-dark-400">已选择 <span class="text-accent-blue font-bold">{{ selectedIds.size }}</span> 项</span>
      <span class="text-sm text-dark-400">共 <span class="text-accent-red font-bold">{{ formatBytes(totalSelectedSize) }}</span></span>
      <button class="btn-primary flex items-center gap-2">
        <Trash2 :size="16" />
        清理选中项
      </button>
    </div>
  </div>
</template>
