<script setup lang="ts">
  import { Check, Copy, Layers } from 'lucide-vue-next';
  import type { Data } from '~/types/data';

  const props = defineProps<{
    source: Data;
    baseUrl: string;
    xyzExpanded: boolean;
    copiedUrl: string | null;
  }>();

  const emit = defineEmits<{
    toggleXyz: [dataId: string];
    copyUrl: [url: string];
  }>();

  function handleToggleXyz() {
    emit('toggleXyz', props.source.id);
  }

  function handleCopy() {
    emit('copyUrl', `${props.baseUrl}/data/${props.source.id}/{z}/{x}/{y}.pbf`);
  }

  const xyzUrl = computed(
    () => `${props.baseUrl}/data/${props.source.id}/{z}/{x}/{y}.pbf`,
  );

  const isCopied = computed(() => props.copiedUrl === xyzUrl.value);
</script>

<template>
  <div
    class="group border border-border bg-background p-4 transition-colors hover:bg-muted/30"
  >
    <div class="flex items-start gap-4">
      <div
        class="flex size-12 shrink-0 items-center justify-center bg-muted ring-1 ring-border"
      >
        <Layers class="size-6 text-muted-foreground" />
      </div>

      <div class="min-w-0 flex-1">
        <div class="flex items-start justify-between gap-2">
          <div>
            <h3 class="font-semibold">{{ source.name || source.id }}</h3>
            <p
              class="mt-0.5 flex flex-wrap items-center gap-2 text-sm text-muted-foreground"
            >
              <code class="bg-muted px-1.5 py-0.5 font-mono text-xs">{{
                source.id
              }}</code>
              <Badge variant="outline" class="text-[10px]">
                z{{ source.minzoom }}-{{ source.maxzoom }}
              </Badge>
            </p>
          </div>

          <Button
            v-if="source.vector_layers?.length"
            as-child
            variant="secondary"
            size="sm"
          >
            <NuxtLink :to="`/data/${source.id}/`">
              <Layers class="mr-1.5 size-4" />
              Inspect
            </NuxtLink>
          </Button>
        </div>

        <template v-if="source.vector_layers?.length">
          <div class="mt-3 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs">
            <span class="text-muted-foreground">Services:</span>
            <a
              :href="`/data/${source.id}.json`"
              target="_blank"
              class="text-primary hover:underline"
              >TileJSON</a
            >
            <span class="text-muted-foreground/30">&bull;</span>
            <button
              class="text-primary hover:underline"
              @click="handleToggleXyz"
            >
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
        </template>
      </div>
    </div>
  </div>
</template>
