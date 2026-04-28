<script setup lang="ts">
import TitleBar from "./components/TitleBar.vue";
</script>

<template>
  <div class="flex flex-col h-screen bg-dark-950 transition-colors duration-200">
    <TitleBar />
    <div class="flex flex-1 overflow-hidden">
      <router-view />
    </div>
  </div>
</template>
