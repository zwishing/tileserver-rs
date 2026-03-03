<script setup lang="ts">
  import { ChevronRight, ExternalLink } from 'lucide-vue-next';
</script>

<template>
  <a href="/_openapi" target="_blank" class="block">
    <Card
      class="overflow-hidden border-border bg-card transition-colors hover:bg-muted/30"
    >
      <div class="flex items-center gap-3 p-4">
        <div class="flex size-8 items-center justify-center bg-muted/50">
          <ExternalLink class="size-4 text-muted-foreground" />
        </div>
        <div class="flex-1">
          <span class="font-medium">API Documentation</span>
          <p class="text-xs text-muted-foreground">
            OpenAPI 3.1 specification with Swagger UI
          </p>
        </div>
        <ChevronRight class="size-4 text-muted-foreground" />
      </div>
    </Card>
  </a>
</template>
