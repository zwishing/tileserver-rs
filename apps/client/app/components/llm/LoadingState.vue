<script setup lang="ts">
  import { BrainCircuit } from 'lucide-vue-next';

  defineProps<{
    stageText: string;
    progress: number;
    modelName: string;
    modelSize: number;
  }>();
</script>

<template>
  <div class="flex flex-col items-center justify-center px-6 py-10">
    <!-- Pulsing icon -->
    <div class="relative mb-5">
      <span class="absolute inset-0 animate-ping rounded-full bg-primary/20" ></span>
      <div class="relative rounded-full bg-primary/10 p-3.5">
        <BrainCircuit class="size-6 text-primary" />
      </div>
    </div>

    <!-- Model info -->
    <p class="mb-1 text-sm font-medium">Loading AI Model</p>
    <p class="mb-5 text-xs text-muted-foreground">
      {{ modelName }} · {{ modelSize }} GB
    </p>

    <!-- Animated progress bar -->
    <div class="w-full max-w-[240px]">
      <div class="relative h-1.5 overflow-hidden rounded-full bg-muted">
        <div
          class="absolute inset-y-0 left-0 rounded-full bg-primary shadow-[0_0_12px_rgba(var(--primary),0.4)] transition-[width] duration-500 ease-out"
          :style="{ width: `${Math.max(progress * 100, 2)}%` }"
        ></div>
      </div>
      <div class="mt-2 flex items-center justify-between">
        <span class="text-[11px] text-muted-foreground">{{ stageText }}</span>
        <span class="text-[11px] font-medium tabular-nums text-muted-foreground">
          {{ Math.round(progress * 100) }}%
        </span>
      </div>
    </div>

    <!-- First-load hint -->
    <p class="mt-6 max-w-[220px] text-center text-[11px] leading-relaxed text-muted-foreground/50">
      First load downloads the model.
      <br />
      Future loads are near-instant.
    </p>
  </div>
</template>
