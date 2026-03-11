<script setup lang="ts">
  const { features } = useMarketingPage();
</script>

<template>
  <section
    data-label="Features"
    class="section-full relative border-b border-border"
  >
    <!-- Geometric grid background -->
    <div
      class="geometric-grid-bg pointer-events-none absolute inset-0 opacity-30"
      aria-hidden="true"
    ></div>

    <div
class="
  relative px-6 py-16
  md:px-12
  lg:px-20
">
      <div class="mb-16">
        <p class="hud-label mb-4">Features</p>
        <h2
          class="
            mb-4 font-display text-3xl leading-[1.15] font-bold
            tracking-[-0.03em]
            sm:text-4xl
          "
        >
          Everything You <span class="text-gradient">Need</span>
        </h2>
        <p
          class="
            mx-auto max-w-xl font-sans text-lg/relaxed text-muted-foreground
          "
        >
          A complete solution for serving vector tiles in production.
        </p>
      </div>

      <div
class="
  grid gap-6
  sm:grid-cols-2
  lg:grid-cols-3
">
        <SpotlightCard
          v-for="feature in features"
          :key="feature.title"
          :border-radius="0"
          spotlight-color="rgba(139, 92, 246, 0.08)"
        >
          <SpotlightCardHeader>
            <div
              class="
                mb-3 inline-flex size-10 items-center justify-center border
                border-primary/20 bg-primary/5 text-primary
              "
            >
              <component :is="feature.icon" class="size-5" />
            </div>
            <SpotlightCardTitle class="text-base">
              {{ feature.title }}
            </SpotlightCardTitle>
          </SpotlightCardHeader>
          <SpotlightCardContent>
            <SpotlightCardDescription>
              {{ feature.description }}
            </SpotlightCardDescription>
          </SpotlightCardContent>
        </SpotlightCard>
      </div>
    </div>
  </section>
</template>
