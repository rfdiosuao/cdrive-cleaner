<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-vue-next";

const appWindow = getCurrentWindow();

async function handleMinimize() {
  await appWindow.minimize();
}

async function handleMaximize() {
  await appWindow.toggleMaximize();
}

async function handleClose() {
  await appWindow.close();
}
</script>

<template>
  <div
    data-tauri-drag-region
    class="flex items-center justify-between h-9 bg-dark-900 border-b border-dark-700 px-3 select-none transition-colors duration-200"
  >
    <div data-tauri-drag-region class="flex items-center gap-2">
      <span class="text-accent-blue font-bold text-sm">C 盘清理助手</span>
      <span class="text-[11px] text-dark-500">0.2.0</span>
    </div>
    <div class="flex items-center gap-1 no-drag">
      <button
        class="flex items-center justify-center w-8 h-7 rounded hover:bg-dark-600 text-dark-400 hover:text-dark-200 transition-colors"
        title="最小化"
        @click="handleMinimize"
      >
        <Minus :size="14" />
      </button>
      <button
        class="flex items-center justify-center w-8 h-7 rounded hover:bg-dark-600 text-dark-400 hover:text-dark-200 transition-colors"
        title="最大化"
        @click="handleMaximize"
      >
        <Square :size="12" />
      </button>
      <button
        class="flex items-center justify-center w-8 h-7 rounded hover:bg-accent-red/80 text-dark-400 hover:text-white transition-colors"
        title="关闭"
        @click="handleClose"
      >
        <X :size="14" />
      </button>
    </div>
  </div>
</template>
