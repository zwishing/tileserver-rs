<script setup lang="ts">
  import {
    Sheet as UiSheet,
    SheetContent as UiSheetContent,
    SheetHeader as UiSheetHeader,
    SheetTitle as UiSheetTitle,
    SheetDescription as UiSheetDescription,
  } from '~/components/ui/sheet';
  import { ScrollArea as UiScrollArea } from '~/components/ui/scroll-area';
  import { Bot, Map, Layers, Search, Globe } from 'lucide-vue-next';
  import type { Map as MaplibreMap } from 'maplibre-gl';

  const props = defineProps<{
    open: boolean;
    mapRef: MaplibreMap | null;
  }>();

  const emit = defineEmits<{
    'update:open': [value: boolean];
  }>();

  const {
    messages,
    isLoading,
    engineStatus,
    loadProgress,
    loadStageText,
    selectedModel,
    availableModels,
    engineError,
    input,
    suggestions,
    showLoadingIndicator,
    scrollAnchor,
    stop,
    handleSubmit,
    handlePromptSelect,
    selectModel,
  } = useLlmPanel(computed(() => props.mapRef));

  const ICON_COMPONENTS = { Map, Layers, Search, Globe } as const;

  function getIconComponent(icon: string) {
    const key = icon.charAt(0).toUpperCase() + icon.slice(1);
    return ICON_COMPONENTS[key as keyof typeof ICON_COMPONENTS] ?? Map;
  }

  function updateOpen(value: boolean) {
    emit('update:open', value);
  }
</script>

<template>
  <UiSheet :open="open" @update:open="updateOpen">
    <UiSheetContent side="right" class="flex w-[400px] flex-col gap-0 border-l border-border/50 p-0 shadow-2xl sm:max-w-[400px]">
      <UiSheetHeader class="space-y-0 border-b border-border/50 bg-muted/30 px-5 py-4">
        <div class="flex items-center justify-between pr-10">
          <UiSheetTitle class="flex items-center gap-2.5 font-display text-base font-semibold tracking-tight">
            <div class="flex size-7 items-center justify-center bg-primary text-primary-foreground">
              <Bot class="size-4" />
            </div>
            Map Assistant
          </UiSheetTitle>
          <span
            v-if="engineStatus === 'ready'"
            class="inline-flex items-center gap-1 border border-primary/30 bg-primary/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-primary"
          >
            <span class="size-1.5 bg-primary" ></span>
            Ready
          </span>
        </div>
        <UiSheetDescription class="mt-2 text-xs">
          <LlmModelSelect
            :models="availableModels"
            :selected-id="selectedModel.id"
            :disabled="engineStatus === 'loading'"
            @select="selectModel"
          />
        </UiSheetDescription>
      </UiSheetHeader>

      <!-- Engine error -->
      <div v-if="engineError" class="border-b border-destructive/20 bg-destructive/5 px-5 py-3">
        <p class="text-xs font-medium text-destructive">{{ engineError }}</p>
      </div>

      <!-- Loading state — centered in messages area -->
      <LlmLoadingState
        v-if="engineStatus === 'loading'"
        :stage-text="loadStageText"
        :progress="loadProgress.progress"
        :model-name="selectedModel.name"
        :model-size="selectedModel.sizeGb"
        class="flex-1"
      />

      <!-- Messages area -->
      <UiScrollArea v-else class="flex-1">
        <!-- Suggested prompts when empty -->
        <div v-if="messages.length === 0" class="flex flex-col gap-3 p-5">
          <p class="text-[11px] font-medium uppercase tracking-widest text-muted-foreground/60">Suggestions</p>
          <div class="grid grid-cols-2 gap-2">
            <button
              v-for="suggestion in suggestions"
              :key="suggestion.title"
              class="group flex flex-col gap-2 border border-border/60 bg-card p-3.5 text-left transition-all duration-200 hover:border-primary/40 hover:bg-primary/5 hover:shadow-sm dark:hover:bg-primary/10"
              @click="handlePromptSelect(suggestion.prompt)"
            >
              <component :is="getIconComponent(suggestion.icon)" class="size-4 text-muted-foreground transition-colors group-hover:text-primary" />
              <span class="text-xs font-medium leading-snug">{{ suggestion.title }}</span>
            </button>
          </div>
        </div>

        <!-- Message list -->
        <LlmMessageList
          v-if="messages.length > 0"
          :messages="messages"
          :is-loading="showLoadingIndicator"
          :is-streaming="isLoading"
        />
        <!-- Scroll anchor for auto-scroll -->
        <div ref="scrollAnchor" class="h-0 w-full" ></div>
      </UiScrollArea>
      <!-- Input -->
      <LlmInput
        v-model="input"
        :is-loading="isLoading"
        :engine-status="engineStatus"
        :has-messages="messages.length > 0"
        @submit="handleSubmit"
        @stop="stop"
      />
    </UiSheetContent>
  </UiSheet>
</template>
