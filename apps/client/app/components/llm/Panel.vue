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
    <UiSheetContent side="right" class="flex w-[380px] flex-col p-0 sm:max-w-[380px]">
      <UiSheetHeader class="border-b px-4 py-3">
        <UiSheetTitle class="flex items-center gap-2 text-base">
          <Bot class="size-4" />
          Map Assistant
          <span
            v-if="engineStatus === 'ready'"
            class="rounded-full bg-green-100 px-2 py-0.5 text-[10px] font-medium text-green-700 dark:bg-green-900/30 dark:text-green-400"
          >
            Ready
          </span>
        </UiSheetTitle>
        <UiSheetDescription class="text-xs">
          <LlmModelSelect
            :models="availableModels"
            :selected-id="selectedModel.id"
            :disabled="engineStatus === 'loading'"
            @select="selectModel"
          />
        </UiSheetDescription>
      </UiSheetHeader>

      <!-- Engine error -->
      <div v-if="engineError" class="border-b bg-destructive/10 px-4 py-3">
        <p class="text-xs text-destructive">{{ engineError }}</p>
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
        <div v-if="messages.length === 0" class="grid grid-cols-2 gap-2 p-4">
          <button
            v-for="suggestion in suggestions"
            :key="suggestion.title"
            class="flex flex-col gap-1.5 rounded-lg border p-3 text-left text-xs transition-colors hover:bg-accent"
            @click="handlePromptSelect(suggestion.prompt)"
          >
            <component :is="getIconComponent(suggestion.icon)" class="size-4 text-muted-foreground" />
            <span class="font-medium">{{ suggestion.title }}</span>
          </button>
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
