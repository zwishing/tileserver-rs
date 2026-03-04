<script setup lang="ts">
  import { Check, Copy, Grid3x3, Image, Map } from 'lucide-vue-next';
  import { motion } from 'motion-v';

  import type { Style } from '~/types/style';

  const props = defineProps<{
    style: Style;
    index: number;
    baseUrl: string;
    isXyzExpanded: boolean;
    copiedUrl: string | null;
  }>();

  const emit = defineEmits<{
    toggleXyz: [styleId: string];
    copyUrl: [url: string];
  }>();

  const xyzUrl = computed(() => `${props.baseUrl}/styles/${props.style.id}/{z}/{x}/{y}.png`);

  function handleToggleXyz() {
    emit('toggleXyz', props.style.id);
  }

  function handleCopyUrl() {
    emit('copyUrl', xyzUrl.value);
  }

  const imgError = ref(false);

  function handleImgError() {
    imgError.value = true;
  }
</script>

<template>
  <motion.div
    :initial="{ opacity: 0, y: 12 }"
    :animate="{ opacity: 1, y: 0 }"
    :transition="{ duration: 0.3, delay: 0.05 * index }"
    class="group rounded-xl border border-border/50 bg-background/50 p-4 transition-all hover:border-primary/30 hover:shadow-lg hover:shadow-primary/5"
  >
    <div class="flex gap-4">
      <!-- Thumbnail -->
      <div class="flex size-20 shrink-0 items-center justify-center overflow-hidden rounded-xl bg-muted ring-1 ring-border/50">
        <img
          v-if="!imgError"
          :src="`/styles/${style.id}/static/0,0,1/160x160.png`"
          :alt="style.name"
          class="size-full object-cover transition-transform group-hover:scale-105"
          loading="lazy"
          @error="handleImgError"
        />
        <Map v-else class="size-8 text-muted-foreground" />
      </div>

      <!-- Content -->
      <div class="min-w-0 flex-1">
        <div class="flex items-start justify-between gap-2">
          <div>
            <h3 class="font-semibold">{{ style.name }}</h3>
            <p class="mt-0.5 text-sm text-muted-foreground">
              <code class="rounded-md bg-muted px-1.5 py-0.5 text-xs font-medium">{{ style.id }}</code>
            </p>
          </div>
          <Button as-child size="sm" class="rounded-lg">
            <NuxtLink :to="`/styles/${style.id}/`">
              <Map class="mr-1.5 size-4" />
              Viewer
            </NuxtLink>
          </Button>
        </div>

        <!-- View modes -->
        <div class="mt-2 flex items-center gap-3">
          <NuxtLink
            :to="`/styles/${style.id}/?raster`"
            class="flex items-center gap-1.5 rounded-lg bg-muted/50 px-2.5 py-1 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <Image class="size-3.5" />
            Raster
          </NuxtLink>
          <NuxtLink
            :to="`/styles/${style.id}/#2/0/0`"
            class="flex items-center gap-1.5 rounded-lg bg-muted/50 px-2.5 py-1 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <Grid3x3 class="size-3.5" />
            Vector
          </NuxtLink>
        </div>

        <!-- Service links -->
        <div class="mt-3 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs">
          <span class="text-muted-foreground">Services:</span>
          <a :href="`/styles/${style.id}/style.json`" target="_blank" class="text-primary hover:underline">GL Style</a>
          <span class="text-muted-foreground/30">•</span>
          <a :href="`/styles/${style.id}.json`" target="_blank" class="text-primary hover:underline">TileJSON</a>
          <span class="text-muted-foreground/30">•</span>
          <a :href="`/styles/${style.id}/wmts.xml`" target="_blank" class="text-primary hover:underline">WMTS</a>
          <span class="text-muted-foreground/30">•</span>
          <button class="text-primary hover:underline" @click="handleToggleXyz">XYZ URL</button>
        </div>

        <!-- XYZ URL expandable -->
        <div v-if="isXyzExpanded" class="mt-2 flex items-center gap-2 rounded-lg bg-muted/50 p-2">
          <code class="flex-1 truncate text-xs text-muted-foreground">{{ xyzUrl }}</code>
          <Button variant="ghost" size="icon" class="size-7 shrink-0 rounded-lg" @click="handleCopyUrl">
            <Check v-if="copiedUrl === xyzUrl" class="size-3.5 text-green-500" />
            <Copy v-else class="size-3.5" />
          </Button>
        </div>
      </div>
    </div>
  </motion.div>
</template>
