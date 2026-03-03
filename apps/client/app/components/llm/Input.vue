<script setup lang="ts">
  import { Send, Square } from 'lucide-vue-next';

  const props = defineProps<{
    modelValue: string;
    isLoading: boolean;
    engineStatus: string;
    hasMessages: boolean;
  }>();

  const emit = defineEmits<{
    'update:modelValue': [value: string];
    submit: [];
    stop: [];
  }>();

  const isDisabled = computed(
    () => props.engineStatus === 'loading' || (!props.modelValue.trim() && !props.isLoading),
  );

  const placeholder = computed(() => {
    if (props.engineStatus === 'loading') return 'Loading model...';
    if (props.engineStatus === 'error') return 'Model failed to load';
    return 'Ask about the map...';
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (!props.isLoading && props.modelValue.trim()) {
        emit('submit');
      }
    }
  }

  function updateValue(e: Event) {
    emit('update:modelValue', (e.target as HTMLInputElement).value);
  }
</script>

<template>
  <form
    class="flex items-end gap-2 border-t bg-background p-3"
    @submit.prevent="isLoading ? emit('stop') : emit('submit')"
  >
    <input
      type="text"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="engineStatus === 'loading' || engineStatus === 'error'"
      class="flex-1 rounded-lg border bg-muted/50 px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:ring-1 focus:ring-ring disabled:opacity-50"
      @input="updateValue"
      @keydown="handleKeydown"
    />
    <button
      type="submit"
      :disabled="isDisabled"
      class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground transition-colors hover:bg-primary/90 disabled:opacity-50"
    >
      <Square v-if="isLoading" class="size-4" />
      <Send v-else class="size-4" />
    </button>
  </form>
</template>
