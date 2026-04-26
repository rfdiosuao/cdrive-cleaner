<script setup lang="ts">
import { Trash2 } from "lucide-vue-next";
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold text-dark-100">执行清理</h1>
      <button class="btn-primary flex items-center gap-2" disabled>
        <Trash2 :size="16" />
        <span>执行清理</span>
      </button>
    </div>

    <div class="card">
      <div class="text-dark-500 text-center py-12">
        <Trash2 :size="48" class="mx-auto mb-4 text-dark-600" />
        <p>暂无清理任务</p>
        <p class="text-sm mt-2">请先扫描后选择要清理的项目</p>
      </div>
    </div>
  </div>
</template>
