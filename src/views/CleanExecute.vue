<script setup lang="ts">
import { computed } from "vue";
import { RouterLink } from "vue-router";
import { CheckCircle2, ShieldCheck, Trash2, Undo2 } from "lucide-vue-next";
import { useScanStore } from "../stores/scan";

const scanStore = useScanStore();

const cleanableCount = computed(() => scanStore.results.length);
const cleanableSize = computed(() => scanStore.results.reduce((sum, item) => sum + item.totalSize, 0));

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-dark-100">执行清理</h1>
        <p class="mt-1 text-sm text-dark-500">清理前会预览风险，并默认要求备份。</p>
      </div>
      <RouterLink to="/scan" class="btn-primary flex items-center gap-2">
        <Trash2 :size="16" />
        <span>选择清理项</span>
      </RouterLink>
    </div>

    <div class="grid grid-cols-3 gap-4">
      <div class="card">
        <div class="text-sm text-dark-400">待处理项目</div>
        <div class="mt-3 text-3xl font-bold">{{ cleanableCount }}</div>
      </div>
      <div class="card">
        <div class="text-sm text-dark-400">可释放空间</div>
        <div class="mt-3 text-3xl font-bold text-accent-red">{{ formatBytes(cleanableSize) }}</div>
      </div>
      <div class="card">
        <div class="text-sm text-dark-400">安全策略</div>
        <div class="mt-3 flex items-center gap-2 text-accent-green">
          <ShieldCheck :size="20" />
          <span class="font-semibold">预览 + 备份</span>
        </div>
      </div>
    </div>

    <div class="card">
      <div v-if="cleanableCount > 0" class="space-y-3">
        <div
          v-for="item in scanStore.results.slice(0, 6)"
          :key="item.id"
          class="flex items-center justify-between rounded-lg bg-dark-900/60 px-4 py-3"
        >
          <div class="flex items-center gap-3">
            <CheckCircle2 :size="18" class="text-accent-green" />
            <div>
              <div class="font-medium">{{ item.target.name }}</div>
              <div class="text-xs text-dark-500">{{ item.fileCount }} 个文件</div>
            </div>
          </div>
          <div class="font-semibold">{{ formatBytes(item.totalSize) }}</div>
        </div>
        <RouterLink to="/scan" class="btn-secondary inline-flex items-center gap-2">
          <Undo2 :size="16" />
          <span>返回扫描结果执行清理</span>
        </RouterLink>
      </div>
      <div v-else class="empty-panel text-dark-500">
        <Trash2 :size="48" class="mx-auto mb-4 text-dark-600" />
        <p>暂无清理任务</p>
        <p class="text-sm mt-2">请先完成扫描，再选择要清理的项目。</p>
      </div>
    </div>
  </div>
</template>
