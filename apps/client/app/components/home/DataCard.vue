<script setup lang="ts">
  import { Check, Copy, Layers } from 'lucide-vue-next';
  import { motion } from 'motion-v';

  import type { Data } from '~/types/data';

  const props = defineProps<{
    source: Data;
    index: number;
    baseUrl: string;
    isXyzExpanded: boolean;
    copiedUrl: string | null;
  }>();

  const emit = defineEmits<{
    toggleXyz: [dataId: string];
    copyUrl: [url: string];
  }>();

  const xyzUrl = computed(() => `${props.baseUrl}/data/${props.source.id}/{z}/{x}/{y}.pbf`);

  function handleToggleXyz() {
    emit('toggleXyz', props.source.id);
  }

  function handleCopyUrl() {
    emit('copyUrl', xyzUrl.value);
  }
</script>

<template>
  <motion.div
    :initial="{ opacity: 0, y: 12 }"
    :animate="{ opacity: 1, y: 0 }"
    :transition="{ duration: 0.3, delay: 0.05 * index }"
    class="group rounded-xl border border-border/50 bg-background/50 p-4 transition-all hover:border-primary/30 hover:shadow-lg hover:shadow-primary/5"
  >
    <div class="flex items-start gap-4">
      <div class="flex size-12 shrink-0 items-center justify-center rounded-xl bg-muted ring-1 ring-border/50">
        <Layers class="size-6 text-muted-foreground" />
      </div>

      <div class="min-w-0 flex-1">
        <div class="flex items-start justify-between gap-2">
          <div>
            <h3 class="font-semibold">{{ source.name || source.id }}</h3>
            <p class="mt-0.5 flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
              <code class="rounded-md bg-muted px-1.5 py-0.5 text-xs font-medium">{{ source.id }}</code>
              <Badge variant="outline" class="rounded-md text-[10px]">
                z{{ source.minzoom }}-{{ source.maxzoom }}
              </Badge>
            </p>
          </div>
          <Button
            v-if="source.vector_layers?.length"
            as-child
            variant="secondary"
            size="sm"
            class="rounded-lg"
          >
            <NuxtLink :to="`/data/${source.id}/`">
              <Layers class="mr-1.5 size-4" />
              Inspect
            </NuxtLink>
          </Button>
        </div>

        <!-- Service links (vector sources only) -->
        <template v-if="source.vector_layers?.length">
          <div class="mt-3 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs">
            <span class="text-muted-foreground">Services:</span>
            <a :href="`/data/${source.id}.json`" target="_blank" class="text-primary hover:underline">TileJSON</a>
            <span class="text-muted-foreground/30">•</span>
            <button class="text-primary hover:underline" @click="handleToggleXyz">XYZ URL</button>
          </div>

          <div v-if="isXyzExpanded" class="mt-2 flex items-center gap-2 rounded-lg bg-muted/50 p-2">
            <code class="flex-1 truncate text-xs text-muted-foreground">{{ xyzUrl }}</code>
            <Button variant="ghost" size="icon" class="size-7 shrink-0 rounded-lg" @click="handleCopyUrl">
              <Check v-if="copiedUrl === xyzUrl" class="size-3.5 text-green-500" />
              <Copy v-else class="size-3.5" />
            </Button>
          </div>
        </template>
      </div>
    </div>
  </motion.div>
</template>
