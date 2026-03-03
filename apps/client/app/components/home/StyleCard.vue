<script setup lang="ts">
  import { Check, Copy, Grid3x3, Image, Map } from 'lucide-vue-next';
  import type { Style } from '~/types/style';

  const props = defineProps<{
    style: Style;
    baseUrl: string;
    xyzExpanded: boolean;
    copiedUrl: string | null;
  }>();

  const emit = defineEmits<{
    toggleXyz: [styleId: string];
    copyUrl: [url: string];
  }>();

  function handleToggleXyz() {
    emit('toggleXyz', props.style.id);
  }

  function handleCopy() {
    emit('copyUrl', `${props.baseUrl}/styles/${props.style.id}/{z}/{x}/{y}.png`);
  }

  const xyzUrl = computed(
    () => `${props.baseUrl}/styles/${props.style.id}/{z}/{x}/{y}.png`,
  );

  const isCopied = computed(() => props.copiedUrl === xyzUrl.value);
</script>

<template>
  <div
    class="group border border-border bg-background p-4 transition-colors hover:bg-muted/30"
  >
    <div class="flex gap-4">
      <div class="size-20 shrink-0 overflow-hidden bg-muted ring-1 ring-border">
        <img
          :src="`/styles/${style.id}/static/0,0,1/160x160.png`"
          :alt="style.name"
          class="size-full object-cover"
          loading="lazy"
        />
      </div>

      <div class="min-w-0 flex-1">
        <div class="flex items-start justify-between gap-2">
          <div>
            <h3 class="font-semibold">{{ style.name }}</h3>
            <p class="mt-0.5 text-sm text-muted-foreground">
              <code class="bg-muted px-1.5 py-0.5 font-mono text-xs">{{
                style.id
              }}</code>
            </p>
          </div>
          <Button as-child size="sm">
            <NuxtLink :to="`/styles/${style.id}/`">
              <Map class="mr-1.5 size-4" />
              Viewer
            </NuxtLink>
          </Button>
        </div>

        <div class="mt-2 flex items-center gap-3">
          <NuxtLink
            :to="`/styles/${style.id}/?raster`"
            class="flex items-center gap-1.5 bg-muted/50 px-2.5 py-1 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <Image class="size-3.5" />
            Raster
          </NuxtLink>
          <NuxtLink
            :to="`/styles/${style.id}/#2/0/0`"
            class="flex items-center gap-1.5 bg-muted/50 px-2.5 py-1 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <Grid3x3 class="size-3.5" />
            Vector
          </NuxtLink>
        </div>

        <div class="mt-3 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs">
          <span class="text-muted-foreground">Services:</span>
          <a
            :href="`/styles/${style.id}/style.json`"
            target="_blank"
            class="text-primary hover:underline"
            >GL Style</a
          >
          <span class="text-muted-foreground/30">&bull;</span>
          <a
            :href="`/styles/${style.id}.json`"
            target="_blank"
            class="text-primary hover:underline"
            >TileJSON</a
          >
          <span class="text-muted-foreground/30">&bull;</span>
          <a
            :href="`/styles/${style.id}/wmts.xml`"
            target="_blank"
            class="text-primary hover:underline"
            >WMTS</a
          >
          <span class="text-muted-foreground/30">&bull;</span>
          <button class="text-primary hover:underline" @click="handleToggleXyz">
            XYZ URL
          </button>
        </div>

        <div
          v-if="xyzExpanded"
          class="mt-2 flex items-center gap-2 bg-muted/50 p-2"
        >
          <code
            class="flex-1 truncate font-mono text-xs text-muted-foreground"
            >{{ xyzUrl }}</code
          >
          <Button
            variant="ghost"
            size="icon"
            class="size-7 shrink-0"
            @click="handleCopy"
          >
            <Check v-if="isCopied" class="size-3.5 text-green-500" />
            <Copy v-else class="size-3.5" />
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
