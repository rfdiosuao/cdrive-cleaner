<script setup lang="ts">
import { RouterLink } from "vue-router";
import { FileSearch, HardDrive, ShieldAlert } from "lucide-vue-next";
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-dark-100">大文件管理</h1>
        <p class="mt-1 text-sm text-dark-500">定位 100MB 以上文件，优先提供迁移和人工确认。</p>
      </div>
      <RouterLink to="/scan" class="btn-primary flex items-center gap-2">
        <FileSearch :size="16" />
        <span>开始扫描</span>
      </RouterLink>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div class="card">
        <HardDrive :size="24" class="text-accent-blue" />
        <h2 class="mt-4 font-semibold">只做建议，不自动删除</h2>
        <p class="mt-2 text-sm text-dark-500">大文件通常包含安装包、视频、虚拟机镜像或项目归档，默认标记为需确认。</p>
      </div>
      <div class="card">
        <ShieldAlert :size="24" class="text-accent-yellow" />
        <h2 class="mt-4 font-semibold">避开系统关键目录</h2>
        <p class="mt-2 text-sm text-dark-500">Windows、Program Files、ProgramData 等目录会被后端安全策略过滤。</p>
      </div>
    </div>

    <div class="card">
      <div class="empty-panel text-dark-500">
        <HardDrive :size="48" class="mx-auto mb-4 text-dark-600" />
        <p>暂无大文件数据</p>
        <p class="text-sm mt-2">扫描完成后将展示 C 盘大文件列表。</p>
      </div>
    </div>
  </div>
</template>
