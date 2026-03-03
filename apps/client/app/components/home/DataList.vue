<script setup lang="ts">
  import { ChevronRight, Database } from 'lucide-vue-next';
  import { motion } from 'motion-v';

  import type { Data } from '~/types/data';

  defineProps<{
    sources: Data[];
    isLoading: boolean;
    hasData: boolean;
    isOpen: boolean;
    searchQuery: string;
    baseUrl: string;
    expandedXyz: Set<string>;
    copiedUrl: string | null;
  }>();

  const emit = defineEmits<{
    'update:isOpen': [value: boolean];
    toggleXyz: [dataId: string];
    copyUrl: [url: string];
  }>();

  function handleToggleXyz(dataId: string) {
    emit('toggleXyz', dataId);
  }

  function handleCopyUrl(url: string) {
    emit('copyUrl', url);
  }
</script>

<template>
  <motion.div
    :initial="{ opacity: 0, y: 12 }"
    :animate="{ opacity: 1, y: 0 }"
    :transition="{ duration: 0.35, delay: 0.25 }"
  >
    <Collapsible :open="isOpen" @update:open="$emit('update:isOpen', $event)">
      <Card class="overflow-hidden rounded-xl border-border/50 bg-card/50 backdrop-blur-sm">
        <CollapsibleTrigger class="flex w-full items-center gap-3 p-4 transition-colors hover:bg-muted/30">
          <ChevronRight class="size-4 text-muted-foreground transition-transform duration-200" :class="{ 'rotate-90': isOpen }" />
          <div class="flex size-8 items-center justify-center rounded-lg bg-primary/10">
            <Database class="size-4 text-primary" />
          </div>
          <span class="font-medium">Data Sources</span>
          <Badge variant="secondary" class="ml-auto rounded-lg">{{ sources.length }}</Badge>
        </CollapsibleTrigger>

        <CollapsibleContent>
          <Separator class="bg-border/50" />
          <div class="p-4">
            <!-- Loading -->
            <div v-if="isLoading" class="flex justify-center py-12">
              <div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary" ></div>
            </div>

            <!-- Empty -->
            <div v-else-if="!hasData" class="py-12 text-center">
              <div class="mx-auto mb-4 flex size-16 items-center justify-center rounded-2xl bg-muted/50">
                <Database class="size-8 text-muted-foreground" />
              </div>
              <p class="font-medium">No data sources configured</p>
              <p class="mt-1 text-sm text-muted-foreground">Add PMTiles or MBTiles to config.toml</p>
            </div>

            <!-- No results -->
            <div v-else-if="sources.length === 0" class="py-12 text-center text-muted-foreground">
              No data sources match "{{ searchQuery }}"
            </div>

            <!-- List -->
            <div v-else class="space-y-3">
              <HomeDataCard
                v-for="(source, i) in sources"
                :key="source.id"
                :source="source"
                :index="i"
                :base-url="baseUrl"
                :is-xyz-expanded="expandedXyz.has(source.id)"
                :copied-url="copiedUrl"
                @toggle-xyz="handleToggleXyz"
                @copy-url="handleCopyUrl"
              />
            </div>
          </div>
        </CollapsibleContent>
      </Card>
    </Collapsible>
  </motion.div>
</template>
