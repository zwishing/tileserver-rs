<script setup lang="ts">
  import { FileJson2, ExternalLink } from 'lucide-vue-next';

  const { apiEndpoints } = useMarketingPage();
</script>

<template>
  <section class="border-t border-border py-20">
    <div class="mx-auto max-w-6xl px-6">
      <div class="mb-16 text-center">
        <p class="hud-label mb-4">API Reference</p>
        <h2
          class="
            mb-4 font-display text-3xl leading-[1.15] font-bold
            tracking-[-0.03em]
            sm:text-4xl
          "
        >
          RESTful <span class="text-gradient">API</span>
        </h2>
        <p
          class="
            mx-auto max-w-xl font-sans text-lg/relaxed text-muted-foreground
          "
        >
          Simple, standards-compliant API with a built-in OpenAPI spec — something neither tileserver-gl nor martin offer.
        </p>
      </div>

      <!-- OpenAPI callout -->
      <div class="mx-auto mb-16 max-w-2xl border border-primary/30 bg-primary/5 p-6">
        <div class="flex items-start gap-4">
          <FileJson2 class="mt-0.5 size-6 shrink-0 text-primary" />
          <div class="flex-1">
            <h3 class="mb-1 font-display text-base font-semibold tracking-tight">
              Interactive OpenAPI Spec
            </h3>
            <p class="mb-3 font-sans text-sm leading-relaxed text-muted-foreground">
              Every endpoint fully documented with request/response schemas. Generate client SDKs, import into Postman, or browse the Scalar UI.
            </p>
            <NuxtLink
              to="https://demo.tileserver.app/_openapi/"
              external
              class="group inline-flex items-center gap-1.5 font-mono text-xs uppercase tracking-wider text-primary hover:text-primary/80"
            >
              Explore the API
              <ExternalLink class="size-3 transition-transform group-hover:translate-x-0.5" />
            </NuxtLink>
          </div>
        </div>
      </div>

      <div
class="
  grid gap-6
  sm:grid-cols-2
">
        <div
          v-for="group in apiEndpoints"
          :key="group.title"
          class="border border-border bg-background p-6"
        >
          <h3
            class="
              mb-4 font-mono text-sm font-semibold tracking-wider
              text-foreground uppercase
            "
          >
            {{ group.title }}
          </h3>
          <div class="space-y-2">
            <div
              v-for="endpoint in group.endpoints"
              :key="endpoint.path"
              class="
                flex items-center gap-3 border border-border bg-muted/30 px-3
                py-2 font-mono text-sm
              "
            >
              <span
                class="
                  bg-emerald-500/20 px-1.5 py-0.5 text-xs font-medium
                  text-emerald-400
                "
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
