<script setup lang="ts">
  import { Bot, X, Map, Layers, Search, Globe } from 'lucide-vue-next';
  import type { Map as MaplibreMap } from 'maplibre-gl';
  import type { OverlayLayer } from '~/types/file-upload';

  const props = defineProps<{
    open: boolean;
    mapRef: MaplibreMap | null;
    overlays: OverlayLayer[];
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
    showLoadingIndicator,
    scrollAnchor,
    stop,
    handleSubmit,
    handlePromptSelect,
    selectModel,
    suggestions,
  } = useLlmPanel(
    computed(() => props.mapRef),
    computed(() => props.overlays),
  );

  const panelRef = ref<HTMLElement | null>(null);
  const ICON_COMPONENTS = { Map, Layers, Search, Globe } as const;

  function getIconComponent(icon: string) {
    const key = icon.charAt(0).toUpperCase() + icon.slice(1);
    return ICON_COMPONENTS[key as keyof typeof ICON_COMPONENTS] ?? Map;
  }

  function close() {
    emit('update:open', false);
  }

  // Auto-focus input and restore scroll position when palette opens
  watch(
    () => props.open,
    (isOpen) => {
      if (isOpen) {
        nextTick(() => {
          panelRef.value?.querySelector<HTMLInputElement>('input')?.focus();
          // ScrollArea resets on v-if remount — scroll to bottom after it initializes
          nextTick(() => scrollAnchor.value?.scrollIntoView({ block: 'end' }));
        });
      }
    },
  );
</script>

<template>
  <!-- Scrim backdrop -->
  <Transition
    enter-active-class="duration-200 ease-out"
    enter-from-class="opacity-0"
    leave-active-class="duration-150 ease-in"
    leave-to-class="opacity-0"
  >
    <div
      v-if="open"
      class="fixed inset-0 z-40 bg-black/10"
      @click="close"
    ></div>
  </Transition>

  <!-- Command palette panel -->
  <Transition
    enter-active-class="duration-200 ease-out"
    enter-from-class="translate-y-4 scale-95 opacity-0"
    leave-active-class="duration-150 ease-in"
    leave-to-class="translate-y-4 scale-95 opacity-0"
  >
    <div
      v-if="open"
      ref="panelRef"
      class="fixed inset-x-0 bottom-6 z-50 mx-auto flex max-h-[60vh] w-[min(640px,calc(100vw-2rem))] flex-col border border-border bg-background/95 shadow-2xl backdrop-blur-xl"
    >
      <!-- Header: status + model select + close -->
      <div
        class="flex items-center justify-between border-b border-border/50 bg-muted/30 px-4 py-2"
      >
        <div class="flex items-center gap-2">
          <Bot class="size-3.5 text-muted-foreground" />
          <span class="text-xs font-semibold tracking-tight">Map AI</span>
          <span
            v-if="engineStatus === 'ready'"
            class="inline-flex items-center gap-1 border border-primary/30 bg-primary/10 px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wider text-primary"
          >
            <span class="size-1 bg-primary"></span>
            Ready
          </span>
        </div>
        <div class="flex items-center gap-2">
          <LlmModelSelect
            :models="availableModels"
            :selected-id="selectedModel.id"
            :disabled="engineStatus === 'loading'"
            class="w-40"
            @select="selectModel"
          />
          <button
            class="flex size-7 items-center justify-center text-muted-foreground transition-colors hover:text-foreground"
            @click="close"
          >
            <X class="size-3.5" />
          </button>
        </div>
      </div>

      <!-- Engine error -->
      <div
        v-if="engineError"
        class="border-b border-destructive/20 bg-destructive/5 px-4 py-2"
      >
        <p class="text-xs text-destructive">{{ engineError }}</p>
      </div>

      <!-- Loading state -->
      <LlmLoadingState
        v-if="engineStatus === 'loading'"
        :stage-text="loadStageText"
        :progress="loadProgress.progress"
        :model-name="selectedModel.name"
        :model-size="selectedModel.sizeGb"
        class="flex-1"
      />

      <!-- Content: suggestions or messages -->
      <div v-else class="min-h-0 flex-1 overflow-y-auto">
        <div v-if="messages.length === 0" class="flex flex-col gap-3 p-4">
          <p
            class="text-[10px] font-medium uppercase tracking-widest text-muted-foreground/50"
          >
            Try asking
          </p>
          <div class="grid grid-cols-2 gap-2">
            <button
              v-for="s in suggestions"
              :key="s.title"
              class="group flex flex-col gap-1.5 border border-border/50 bg-card p-3 text-left transition-all hover:border-primary/30 hover:bg-primary/5"
              @click="handlePromptSelect(s.prompt)"
            >
              <component
                :is="getIconComponent(s.icon)"
                class="size-3.5 text-muted-foreground transition-colors group-hover:text-primary"
              />
              <span class="text-[11px] font-medium leading-snug">{{
                s.title
              }}</span>
            </button>
          </div>
        </div>
        <LlmMessageList
          v-if="messages.length > 0"
          :messages="messages"
          :is-loading="showLoadingIndicator"
          :is-streaming="isLoading"
        />
        <div ref="scrollAnchor" class="size-0"></div>
      </div>

      <!-- Input -->
      <LlmInput
        v-model="input"
        :is-loading="isLoading"
        :engine-status="engineStatus"
        :has-messages="messages.length > 0"
        @submit="handleSubmit"
        @stop="stop"
      />
    </div>
  </Transition>
</template>
