<script setup lang="ts">
import { useRoute } from "vue-router";
import {
  LayoutDashboard,
  ScanSearch,
  Trash2,
  HardDrive,
  Package,
  Brain,
  Settings,
} from "lucide-vue-next";

const route = useRoute();

const navItems = [
  { path: "/", label: "仪表盘", icon: LayoutDashboard },
  { path: "/scan", label: "扫描结果", icon: ScanSearch },
  { path: "/clean", label: "执行清理", icon: Trash2 },
  { path: "/large-file", label: "大文件管理", icon: HardDrive },
  { path: "/software-move", label: "软件迁移", icon: Package },
  { path: "/ai-migrate", label: "智能迁移", icon: Brain },
  { path: "/settings", label: "设置", icon: Settings },
];
</script>

<template>
  <aside class="w-56 flex-shrink-0 bg-dark-900 border-r border-dark-700 flex flex-col">
    <nav class="flex-1 py-4 px-3 space-y-1">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors"
        :class="
          route.path === item.path
            ? 'bg-accent-blue/15 text-accent-blue'
            : 'text-dark-400 hover:bg-dark-800 hover:text-dark-200'
        "
      >
        <component :is="item.icon" :size="18" />
        <span>{{ item.label }}</span>
      </router-link>
    </nav>
    <div class="p-3 border-t border-dark-700">
      <div class="text-xs text-dark-500 text-center">v0.1.0</div>
    </div>
  </aside>
</template>
