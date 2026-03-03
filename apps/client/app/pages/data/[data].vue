<script setup lang="ts">
  import { VMap, VControlNavigation, VControlScale } from '@geoql/v-maplibre';
  import {
    ArrowLeft,
    Eye,
    EyeOff,
    Layers,
    PanelRightClose,
    PanelRightOpen,
  } from 'lucide-vue-next';
  import { motion, AnimatePresence } from 'motion-v';

  const route = useRoute('data-data');
  const dataId = computed(() => String(route.params.data));
  const {
    mapOptions,
    layerColors,
    panelOpen,
    navigateBack,
    togglePanel,
    toggleLayerVisibility,
    onMapLoaded,
  } = useDataInspector(dataId);
</script>

<template>
  <div class="relative h-dvh w-full">
    <!-- Map (full screen) -->
    <ClientOnly>
      <VMap :options="mapOptions" class="size-full" @loaded="onMapLoaded">
        <VControlNavigation position="bottom-right" />
        <VControlScale position="bottom-left" />
      </VMap>
    </ClientOnly>

    <!-- Back button -->
    <button
      class="absolute top-4 left-4 z-10 flex items-center gap-2 border border-border bg-background px-3 py-2 text-sm font-medium shadow-sm transition-colors hover:bg-accent"
      @click="navigateBack"
    >
      <ArrowLeft class="size-4" />
      <Layers class="size-4" />
      <span>{{ dataId }}</span>
    </button>

    <!-- Panel toggle button -->
    <button
      class="absolute top-4 right-4 z-10 flex size-9 items-center justify-center border border-border shadow-sm transition-colors hover:bg-accent"
      :class="
        panelOpen
          ? 'bg-background/95'
          : 'bg-primary text-primary-foreground hover:bg-primary/90'
      "
      @click="togglePanel"
    >
      <PanelRightClose v-if="panelOpen" class="size-4" />
      <PanelRightOpen v-else class="size-4" />
    </button>

    <!-- Collapsible layers panel -->
    <AnimatePresence>
      <motion.div
        v-if="panelOpen"
        :initial="{ opacity: 0, x: 20, scale: 0.95 }"
        :animate="{ opacity: 1, x: 0, scale: 1 }"
        :exit="{ opacity: 0, x: 20, scale: 0.95 }"
        :transition="{ type: 'spring', stiffness: 300, damping: 25 }"
        class="absolute top-16 right-4 z-10 w-56 border border-border bg-background p-4 shadow-sm"
      >
        <h3 class="mb-3 text-sm font-semibold">Layers</h3>
        <div class="space-y-1">
          <button
            v-for="layer in layerColors"
            :key="layer.id"
            class="flex w-full items-center gap-2 px-1.5 py-1 text-sm transition-colors hover:bg-accent"
            :class="{ 'opacity-40': !layer.visible }"
            @click="toggleLayerVisibility(layer.id)"
          >
            <div
              class="size-3.5 shrink-0"
              :style="{ backgroundColor: layer.color }"
            ></div>
            <span
              class="flex-1 truncate text-left text-muted-foreground"
              :class="{ 'line-through': !layer.visible }"
            >
              {{ layer.id }}
            </span>
            <Eye
              v-if="layer.visible"
              class="size-3.5 shrink-0 text-muted-foreground"
            />
            <EyeOff v-else class="size-3.5 shrink-0 text-muted-foreground" />
          </button>
          <div
            v-if="layerColors.length === 0"
            class="px-1.5 py-1 text-sm text-muted-foreground"
          >
            Loading layers...
          </div>
        </div>
      </motion.div>
    </AnimatePresence>
  </div>
</template>
