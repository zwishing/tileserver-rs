<script setup lang="ts">
  import {
    Select as UiSelect,
    SelectContent as UiSelectContent,
    SelectItem as UiSelectItem,
    SelectTrigger as UiSelectTrigger,
    SelectValue as UiSelectValue,
  } from '~/components/ui/select';
  import type { LlmModelConfig } from '~/types/llm';

  defineProps<{
    models: LlmModelConfig[];
    selectedId: string;
    disabled: boolean;
  }>();

  const emit = defineEmits<{
    select: [modelId: string];
  }>();

  function handleChange(value: string) {
    emit('select', value);
  }
</script>

<template>
  <UiSelect :model-value="selectedId" :disabled="disabled" @update:model-value="handleChange">
    <UiSelectTrigger class="h-7 w-full gap-1 border-none bg-transparent px-2 text-[11px] shadow-none hover:bg-accent">
      <UiSelectValue placeholder="Select model" />
    </UiSelectTrigger>
    <UiSelectContent align="end" class="min-w-[240px]">
      <UiSelectItem
        v-for="model in models"
        :key="model.id"
        :value="model.id"
        class="text-xs"
      >
        <span class="flex items-center gap-1.5">
          {{ model.name }}
          <span class="text-muted-foreground">({{ model.sizeGb }}GB)</span>
        </span>
      </UiSelectItem>
    </UiSelectContent>
  </UiSelect>
</template>
