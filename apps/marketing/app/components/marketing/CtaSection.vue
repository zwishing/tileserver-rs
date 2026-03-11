<script setup lang="ts">
  import { Github, ArrowRight } from 'lucide-vue-next';
</script>

<template>
  <section
    data-label="Get Started"
    class="section-full relative"
  >
    <div
      class="geometric-grid-bg pointer-events-none absolute inset-0 opacity-30"
      aria-hidden="true"
    />

    <div class="relative px-6 py-16 md:px-12 lg:px-20 text-center">
      <div class="mx-auto max-w-2xl">
        <p class="hud-label mb-4">Get Started</p>
        <h2
          class="
            mb-4 font-display text-3xl leading-[1.15] font-bold
            tracking-[-0.03em]
            sm:text-4xl
          "
        >
          Ready to <span class="text-gradient">Deploy</span>?
        </h2>
        <p class="mb-10 font-sans text-lg/relaxed text-muted-foreground">
          Deploy your own tile server in minutes with our comprehensive
          documentation.
        </p>
        <div
          class="
            flex flex-col items-center justify-center gap-3
            sm:flex-row
          "
        >
          <Button
            size="lg"
            as="a"
            href="https://docs.tileserver.app/getting-started/quickstart"
            class="
              group gap-2 bg-primary px-8 text-primary-foreground
              hover:bg-primary/90
            "
          >
            Read the Docs
            <ArrowRight
              class="
                size-4 transition-transform
                group-hover:translate-x-0.5
              "
            />
          </Button>
          <Button
            variant="outline"
            size="lg"
            as="a"
            href="https://github.com/vinayakkulkarni/tileserver-rs"
            class="
              gap-2 border-border bg-transparent
              hover:border-foreground/20 hover:bg-accent
            "
          >
            <Github class="size-4" />
            Star on GitHub
          </Button>
        </div>
      </div>
    </div>
  </section>
</template>
