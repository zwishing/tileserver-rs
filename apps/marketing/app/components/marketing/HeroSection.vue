<script setup lang="ts">
  import {
    Github,
    Sparkles,
    ArrowRight,
    ExternalLink,
    Terminal,
    Copy,
    Check,
  } from 'lucide-vue-next';

  const { copied, installCommand, copyToClipboard } = useMarketingPage();
</script>

<template>
  <section class="relative pt-28 pb-20">
    <div class="mx-auto max-w-6xl px-6">
      <div class="mx-auto max-w-4xl text-center">
        <Badge
          variant="outline"
          class="mb-8 gap-2 border-border bg-muted/50 px-4 py-1.5 font-mono text-xs tracking-wider uppercase backdrop-blur-sm"
        >
          <span class="status-online"></span>
          <span class="text-muted-foreground">Available on</span>
          <a
            href="https://github.com/vinayakkulkarni/tileserver-rs/releases"
            class="text-primary hover:underline"
          >
            GitHub Releases
          </a>
        </Badge>

        <h1
          class="mb-8 font-display text-5xl leading-[1.1] font-bold tracking-[-0.04em] sm:text-6xl lg:text-7xl"
        >
          <span class="text-foreground">Serve Vector Tiles</span>
          <ShinyText
            text="At Lightning Speed"
            :speed="4"
            class="mt-2 block pb-2 font-display leading-[1.1]"
          />
        </h1>

        <SplitText
          text="High-performance tile server built in Rust. Serve PMTiles, MBTiles, PostGIS, and Cloud Optimized GeoTIFF with native MapLibre rendering for static images."
          by="words"
          :delay="30"
          :duration="0.4"
          class="mx-auto mb-10 max-w-2xl justify-center font-sans text-lg/relaxed text-muted-foreground sm:text-xl"
        />

        <div
          class="flex flex-col items-center justify-center gap-3 sm:flex-row"
        >
          <Button
            size="lg"
            as="a"
            href="https://docs.tileserver.app/getting-started/quickstart"
            class="group gap-2 bg-primary px-8 text-primary-foreground hover:bg-primary/90"
          >
            <Sparkles class="size-4" />
            Get Started
            <ArrowRight
              class="size-4 transition-transform group-hover:translate-x-0.5"
            />
          </Button>
          <Button
            variant="outline"
            size="lg"
            as="a"
            href="https://github.com/vinayakkulkarni/tileserver-rs"
            class="gap-2 border-border bg-transparent hover:border-foreground/20 hover:bg-accent"
          >
            <Github class="size-4" />
            View on GitHub
          </Button>
          <Button
            variant="outline"
            size="lg"
            as="a"
            href="https://demo.tileserver.app"
            target="_blank"
            class="gap-2 border-emerald-500/20 bg-emerald-500/5 text-emerald-400 hover:border-emerald-500/40 hover:bg-emerald-500/10"
          >
            <ExternalLink class="size-4" />
            Live Demo
          </Button>
        </div>

        <div class="mx-auto mt-14 max-w-xl">
          <div
            class="flex items-center gap-4 border border-border bg-muted/30 p-4"
          >
            <Terminal class="size-4 shrink-0 text-primary" />
            <code class="flex-1 text-left font-mono text-sm text-foreground">
              {{ installCommand }}
            </code>
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger as-child>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    class="text-muted-foreground hover:text-primary"
                    @click="copyToClipboard()"
                  >
                    <Check v-if="copied" class="size-4 text-emerald-400" />
                    <Copy v-else class="size-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent class="border-border bg-card">
                  <p>{{ copied ? 'Copied!' : 'Copy to clipboard' }}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
