<script setup lang="ts">
  import { ArrowRight } from 'lucide-vue-next';

  const { performanceStats } = useMarketingPage();
</script>

<template>
  <section
    data-label="Performance"
    class="section-full relative border-b border-border"
  >
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
        <p class="hud-label mb-4">Performance</p>
        <h2
          class="
            mb-4 font-display text-3xl leading-[1.15] font-bold
            tracking-[-0.03em]
            sm:text-4xl
          "
        >
          Benchmarked &amp; <span class="text-gradient">Proven</span>
        </h2>
        <p
          class="
            mx-auto max-w-xl font-sans text-lg/relaxed text-muted-foreground
          "
        >
          Real numbers from Docker-to-Docker benchmarks on ARM64. No marketing
          fluff.
        </p>
      </div>

      <div
class="
  grid gap-6
  sm:grid-cols-2
  lg:grid-cols-4
">
        <div
          v-for="stat in performanceStats"
          :key="stat.label"
          class="border border-border bg-background p-8 text-center"
        >
          <div
            class="
              mx-auto mb-3 inline-flex size-10 items-center justify-center
              border border-primary/20 bg-primary/5 text-primary
            "
          >
            <component :is="stat.icon" class="size-5" />
          </div>
          <p class="font-display text-3xl font-bold text-primary tabular-nums">
            <span v-if="stat.prefix">{{ stat.prefix }}</span>
            <CountUp :to="stat.value" :duration="2.5" />
          </p>
          <p class="mt-1 text-sm text-muted-foreground">{{ stat.label }}</p>
          <p class="mt-1 text-xs text-emerald-400">{{ stat.detail }}</p>
        </div>
      </div>

      <div class="mt-8 text-center">
        <Button
          variant="outline"
          size="sm"
          as="a"
          href="https://docs.tileserver.app/benchmarks/performance"
          class="
            gap-2 border-border bg-transparent font-mono text-xs tracking-wider
            uppercase
            hover:border-foreground/20 hover:bg-accent
          "
        >
          View Full Benchmarks
          <ArrowRight class="size-3.5" />
        </Button>
      </div>
    </div>
  </section>
</template>
