<script setup lang="ts">
import { Brain } from "lucide-vue-next";
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold text-dark-100">智能迁移</h1>

    <div class="card">
      <div class="text-dark-500 text-center py-12">
        <Brain :size="48" class="mx-auto mb-4 text-dark-600" />
        <p>AI智能迁移</p>
        <p class="text-sm mt-2">基于AI分析，智能推荐文件和软件迁移方案</p>
        <p class="text-xs mt-4 text-dark-600">请在设置中配置AI模型后再使用</p>
      </div>
    </div>
  </div>
</template>
