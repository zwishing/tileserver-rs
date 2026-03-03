<script setup lang="ts">
  import { Search } from 'lucide-vue-next';

  const model = defineModel<string>({ required: true });
</script>

<template>
  <div class="relative">
    <Search
      class="absolute top-1/2 left-4 size-4 -translate-y-1/2 text-muted-foreground"
    />
    <Input
      v-model="model"
      placeholder="Search styles and data sources..."
      class="h-11 border-border bg-muted/30 pl-11 transition-all focus:bg-background"
    />
  </div>
</template>
