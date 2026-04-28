<script setup lang="ts">
import { RouterLink } from "vue-router";
import { ArrowRightLeft, Package, Route } from "lucide-vue-next";
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-dark-100">软件迁移</h1>
        <p class="mt-1 text-sm text-dark-500">识别可迁移软件，后续通过目录联接降低 C 盘占用。</p>
      </div>
      <RouterLink to="/scan" class="btn-primary flex items-center gap-2">
        <Package :size="16" />
        <span>扫描软件</span>
      </RouterLink>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div class="card">
        <Route :size="24" class="text-accent-blue" />
        <h2 class="mt-4 font-semibold">迁移路径规划</h2>
        <p class="mt-2 text-sm text-dark-500">展示安装位置、体积和建议目标盘，避免盲目移动程序文件。</p>
      </div>
      <div class="card">
        <ArrowRightLeft :size="24" class="text-accent-green" />
        <h2 class="mt-4 font-semibold">可回滚设计</h2>
        <p class="mt-2 text-sm text-dark-500">迁移任务会保留原路径映射信息，便于之后恢复。</p>
      </div>
    </div>

    <div class="card">
      <div class="empty-panel text-dark-500">
        <Package :size="48" class="mx-auto mb-4 text-dark-600" />
        <p>暂无软件迁移建议</p>
        <p class="text-sm mt-2">扫描完成后将展示可迁移的软件列表。</p>
      </div>
    </div>
  </div>
</template>
