<script setup lang="ts">
import { Package } from "lucide-vue-next";
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold text-dark-100">软件迁移</h1>

    <div class="card">
      <div class="text-dark-500 text-center py-12">
        <Package :size="48" class="mx-auto mb-4 text-dark-600" />
        <p>暂无软件迁移建议</p>
        <p class="text-sm mt-2">扫描完成后将展示可迁移的软件列表</p>
      </div>
    </div>
  </div>
</template>
