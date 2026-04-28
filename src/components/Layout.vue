<script setup lang="ts">
import Sidebar from "./Sidebar.vue";
</script>

<template>
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <main class="flex-1 overflow-y-auto p-6 bg-dark-950 transition-colors duration-200">
      <router-view />
    </main>
  </div>
</template>
