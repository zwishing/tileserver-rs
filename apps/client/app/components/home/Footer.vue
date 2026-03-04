<script setup lang="ts">
  defineProps<{
    versionLabel: string;
  }>();
</script>

<template>
  <footer class="mt-auto border-t border-border/50 py-6">
    <p class="text-center text-sm text-muted-foreground">
      Tileserver RS — Built with Rust + Axum + MapLibre GL JS
    </p>
    <p
      v-if="versionLabel"
      class="mt-1 text-center font-mono text-xs text-muted-foreground/60"
    >
      {{ versionLabel }}
    </p>
  </footer>
</template>
