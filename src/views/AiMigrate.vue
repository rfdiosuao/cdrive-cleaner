<script setup lang="ts">
import { RouterLink } from "vue-router";
import { Brain, Cloud, Cpu } from "lucide-vue-next";
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-dark-100">智能迁移</h1>
        <p class="mt-1 text-sm text-dark-500">结合本地规则和云端模型，生成更谨慎的迁移方案。</p>
      </div>
      <RouterLink to="/settings" class="btn-primary flex items-center gap-2">
        <Brain :size="16" />
        <span>配置 AI</span>
      </RouterLink>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div class="card">
        <Cpu :size="24" class="text-accent-green" />
        <h2 class="mt-4 font-semibold">本地优先</h2>
        <p class="mt-2 text-sm text-dark-500">常规文件分类和安全评分可离线完成，减少敏感路径外发。</p>
      </div>
      <div class="card">
        <Cloud :size="24" class="text-accent-blue" />
        <h2 class="mt-4 font-semibold">云端增强</h2>
        <p class="mt-2 text-sm text-dark-500">当迁移关系复杂时，可接入 OpenAI、DeepSeek、通义千问或自定义接口。</p>
      </div>
    </div>

    <div class="card">
      <div class="empty-panel text-dark-500">
        <Brain :size="48" class="mx-auto mb-4 text-dark-600" />
        <p>AI 智能迁移</p>
        <p class="text-sm mt-2">请在设置中配置 AI 模型，扫描后会生成文件和软件迁移方案。</p>
      </div>
    </div>
  </div>
</template>
