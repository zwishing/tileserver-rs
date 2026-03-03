<script setup lang="ts">
  const { apiEndpoints } = useMarketingPage();
</script>

<template>
  <section class="border-t border-border py-20">
    <div class="mx-auto max-w-6xl px-6">
      <div class="mb-16 text-center">
        <p class="hud-label mb-4">API Reference</p>
        <h2
          class="mb-4 font-display text-3xl leading-[1.15] font-bold tracking-[-0.03em] sm:text-4xl"
        >
          RESTful <span class="text-gradient">API</span>
        </h2>
        <p
          class="mx-auto max-w-xl font-sans text-lg/relaxed text-muted-foreground"
        >
          Simple, standards-compliant API for all your tile serving needs.
        </p>
      </div>

      <div class="grid gap-6 sm:grid-cols-2">
        <div
          v-for="group in apiEndpoints"
          :key="group.title"
          class="border border-border bg-background p-6"
        >
          <h3
            class="mb-4 font-mono text-sm font-semibold tracking-wider text-foreground uppercase"
          >
            {{ group.title }}
          </h3>
          <div class="space-y-2">
            <div
              v-for="endpoint in group.endpoints"
              :key="endpoint.path"
              class="flex items-center gap-3 border border-border bg-muted/30 px-3 py-2 font-mono text-sm"
            >
              <span
                class="bg-emerald-500/20 px-1.5 py-0.5 text-xs font-medium text-emerald-400"
              >
                {{ endpoint.method }}
              </span>
              <span class="text-muted-foreground">{{ endpoint.path }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
