<script setup lang="ts">
  import {
    VMap,
    VControlNavigation,
    VControlScale,
    VControlGeolocate,
  } from '@geoql/v-maplibre';
  import { ArrowLeft, Bot, Palette } from 'lucide-vue-next';

  const route = useRoute('styles-style');
  const styleId = computed(() => String(route.params.style));
  const isRaster = computed(() => 'raster' in route.query);
  const isScreenshot = computed(() => 'screenshot' in route.query);

  const { mapOptions, mapRef, isLoading, navigateBack, onMapLoaded } = useStyleViewer(
    styleId,
    isRaster,
  );

  const chatOpen = ref(false);

  function toggleChat() {
    chatOpen.value = !chatOpen.value;
  }
</script>

<template>
  <div class="relative h-dvh w-full">
    <!-- Floating back button (hidden for screenshots) -->
    <button
      v-if="!isScreenshot"
      class="absolute top-4 left-4 z-10 flex items-center gap-2 border border-border bg-background px-3 py-2 text-sm font-medium shadow-sm transition-colors hover:bg-accent"
      @click="navigateBack"
    >
      <ArrowLeft class="size-4" />
      <Palette class="size-4" />
      <span>{{ styleId }}</span>
    </button>

    <!-- LLM chat toggle (hidden for screenshots) -->
    <button
      v-if="!isScreenshot"
      class="absolute right-4 bottom-8 z-10 flex size-12 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-lg transition-transform hover:scale-105 active:scale-95"
      title="Chat with map"
      @click="toggleChat"
    >
      <Bot class="size-5" />
    </button>

    <!-- Loading -->
    <div
      v-if="isLoading"
      class="flex size-full items-center justify-center bg-slate-100 dark:bg-slate-900"
    >
      <span class="text-slate-500 dark:text-slate-400"> Loading style... </span>
    </div>

    <!-- Map - VMap wrapped in ClientOnly -->
    <div
      v-if="!isLoading && mapOptions"
      class="absolute inset-0 size-full overflow-hidden"
    >
      <ClientOnly>
        <VMap :options="mapOptions" :support-pmtiles="false" class="size-full" @loaded="onMapLoaded">
          <VControlScale v-if="!isScreenshot" position="bottom-left" />
          <VControlNavigation v-if="!isScreenshot" position="bottom-right" />
          <VControlGeolocate v-if="!isScreenshot" position="bottom-right" />
        </VMap>
      </ClientOnly>
    </div>

    <!-- LLM Chat Panel -->
    <ClientOnly>
      <LlmPanel
        :open="chatOpen"
        :map-ref="mapRef"
        @update:open="chatOpen = $event"
      />
    </ClientOnly>
  </div>
</template>
