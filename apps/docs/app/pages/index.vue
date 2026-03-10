<script setup lang="ts">
  import {
    ArrowRight,
    Zap,
    Globe,
    Layers,
    Database,
    Image,
    RefreshCw,
    Github,
    ExternalLink,
    Map,
    Camera,
    Container,
    Sun,
    Moon,
  } from 'lucide-vue-next';

  definePageMeta({ layout: false });

  const { isDark, toggle: toggleTheme } = useThemeToggle();

  const starColor = computed(() =>
    isDark.value ? '#a5b4fc' : '#6366f1',
  );

  const backgroundColor = computed(() =>
    isDark.value ? '#030014' : '#f8fafc',
  );

  usePageSeo({
    title: 'Tileserver RS Docs - High-Performance Vector Tile Server',
    description:
      'Documentation for tileserver-rs — a high-performance vector tile server built in Rust.',
    path: '/',
  });

  const features = [
    {
      icon: Zap,
      title: 'Built in Rust',
      description:
        'High-performance tile serving with the safety and speed of Rust and Axum.',
    },
    {
      icon: Map,
      title: 'PMTiles Support',
      description:
        'Cloud-optimized tile archives with HTTP range request support.',
    },
    {
      icon: Database,
      title: 'MBTiles Support',
      description: 'SQLite-based tile storage for easy local development.',
    },
    {
      icon: Layers,
      title: 'MapLibre GL JS',
      description:
        'Built-in map viewer and data inspector powered by MapLibre GL JS.',
    },
    {
      icon: Globe,
      title: 'TileJSON 3.0',
      description:
        'Full TileJSON metadata API for seamless integration with mapping clients.',
    },
    {
      icon: Image,
      title: 'Native Raster Rendering',
      description:
        'Generate PNG/JPEG/WebP tiles from vector styles using MapLibre Native (~100ms/tile).',
    },
    {
      icon: Camera,
      title: 'Static Map Images',
      description:
        'Create embeddable map screenshots like Mapbox/Maptiler static API.',
    },
    {
      icon: Container,
      title: 'Docker Ready',
      description:
        'Easy deployment with Docker Compose v2 and multi-stage builds.',
    },
    {
      icon: RefreshCw,
      title: 'MLT Transcoding',
      description:
        'On-the-fly MLT↔MVT transcoding — serve next-gen MapLibre Tiles from existing MVT sources.',
    },
  ];

  const sections = [
    {
      title: 'Getting Started',
      path: '/getting-started/installation',
      description: 'Install, configure, and run your first tile server.',
    },
    {
      title: 'API Reference',
      path: '/api/endpoints',
      description: 'Full endpoint documentation for tiles, styles, and metadata.',
    },
    {
      title: 'Guides',
      path: '/guides/static-images',
      description: 'Static images, Docker, PostgreSQL, hot reload, and more.',
    },
    {
      title: 'Benchmarks',
      path: '/benchmarks/performance',
      description: 'Performance benchmarks and comparison with tileserver-gl.',
    },
  ];
</script>

<template>
  <div class="relative min-h-dvh bg-background text-foreground">
    <!-- Galaxy background -->
    <div class="pointer-events-none fixed inset-0">
      <Galaxy
        :speed="0.3"
        :star-count="1500"
        :star-size="2"
        :star-color="starColor"
        :background-color="backgroundColor"
      />
    </div>

    <div class="relative z-10">
    <!-- Nav -->
    <nav
      class="fixed top-0 z-50 w-full border-b border-border bg-background/80 backdrop-blur-md"
    >
      <div
        class="flex h-14 items-center justify-between px-6"
      >
        <NuxtLink
          to="/"
          class="flex items-center gap-2.5"
        >
          <Globe class="size-5 text-primary" />
          <span
            class="font-display text-sm font-semibold uppercase tracking-[0.15em]"
          >
            <span class="text-foreground">Tileserver</span>
            <span class="text-primary"> RS</span>
          </span>
          <span
            class="border border-border bg-muted px-1.5 py-0.5 font-mono text-[10px] uppercase tracking-wider text-muted-foreground"
          >
            docs
          </span>
        </NuxtLink>
        <div class="flex items-center gap-1">
          <NuxtLink
            to="https://tileserver.app"
            external
            class="px-3 py-1.5 font-mono text-xs uppercase tracking-wider text-muted-foreground hover:text-foreground"
          >
            Home
          </NuxtLink>
          <button
            class="flex size-9 items-center justify-center text-muted-foreground transition-colors hover:text-foreground"
            aria-label="Toggle theme"
            @click="toggleTheme"
          >
            <Sun
              v-if="isDark"
              class="size-4"
            />
            <Moon
              v-else
              class="size-4"
            />
          </button>
          <NuxtLink
            to="https://github.com/vinayakkulkarni/tileserver-rs"
            external
            class="flex items-center gap-1.5 border border-border bg-transparent px-3 py-1.5 font-mono text-xs uppercase tracking-wider text-muted-foreground hover:border-foreground/20 hover:text-foreground"
          >
            <Github class="size-3.5" />
            GitHub
          </NuxtLink>
        </div>
      </div>
    </nav>

    <!-- Hero -->
    <section class="pt-32 pb-20">
      <div class="mx-auto max-w-6xl px-6">
        <p
          class="mb-4 font-mono text-xs uppercase tracking-[0.2em] text-muted-foreground"
        >
          Documentation
        </p>
        <h1
          class="mb-6 max-w-3xl font-display text-5xl font-bold md:text-6xl"
          style="letter-spacing: -0.04em; line-height: 1.1"
        >
          Tileserver
          <span class="text-primary">RS</span>
        </h1>
        <p
          class="mb-10 max-w-2xl font-sans text-lg leading-relaxed text-muted-foreground"
        >
          High-performance vector tile server built in Rust. Serve PMTiles and
          MBTiles with native raster rendering, static map images, and TileJSON
          3.0 metadata.
        </p>
        <div class="flex flex-wrap gap-3">
          <NuxtLink
            to="/getting-started/installation"
            class="group inline-flex items-center gap-2 border border-foreground bg-foreground px-5 py-2.5 font-mono text-xs uppercase tracking-wider text-background transition-colors hover:bg-transparent hover:text-foreground"
          >
            Get Started
            <ArrowRight
              class="size-3.5 transition-transform group-hover:translate-x-0.5"
            />
          </NuxtLink>
          <NuxtLink
            to="https://github.com/vinayakkulkarni/tileserver-rs"
            external
            class="inline-flex items-center gap-2 border border-border px-5 py-2.5 font-mono text-xs uppercase tracking-wider text-muted-foreground transition-colors hover:border-foreground/20 hover:text-foreground"
          >
            <Github class="size-3.5" />
            View Source
          </NuxtLink>
          <NuxtLink
            to="https://demo.tileserver.app"
            external
            class="inline-flex items-center gap-2 border border-border px-5 py-2.5 font-mono text-xs uppercase tracking-wider text-muted-foreground transition-colors hover:border-foreground/20 hover:text-foreground"
          >
            <ExternalLink class="size-3.5" />
            Live Demo
          </NuxtLink>
        </div>
      </div>
    </section>

    <!-- Features -->
    <section class="border-t border-border py-20">
      <div class="mx-auto max-w-6xl px-6">
        <p
          class="mb-3 font-mono text-xs uppercase tracking-[0.2em] text-muted-foreground"
        >
          Features
        </p>
        <h2
          class="mb-12 font-display text-3xl font-semibold"
          style="letter-spacing: -0.03em; line-height: 1.15"
        >
          Everything you need to serve tiles
        </h2>
        <div class="grid gap-px border border-border bg-border md:grid-cols-3">
          <div
            v-for="feature in features"
            :key="feature.title"
            class="bg-background p-6"
          >
            <component
              :is="feature.icon"
              class="mb-3 size-5 text-primary"
            />
            <h3 class="mb-2 font-display text-sm font-semibold tracking-tight">
              {{ feature.title }}
            </h3>
            <p class="font-sans text-sm leading-relaxed text-muted-foreground">
              {{ feature.description }}
            </p>
          </div>
        </div>
      </div>
    </section>

    <!-- Quick links -->
    <section class="border-t border-border py-20">
      <div class="mx-auto max-w-6xl px-6">
        <p
          class="mb-3 font-mono text-xs uppercase tracking-[0.2em] text-muted-foreground"
        >
          Explore
        </p>
        <h2
          class="mb-12 font-display text-3xl font-semibold"
          style="letter-spacing: -0.03em; line-height: 1.15"
        >
          Jump into the docs
        </h2>
        <div class="grid gap-4 md:grid-cols-2">
          <NuxtLink
            v-for="section in sections"
            :key="section.path"
            :to="section.path"
            class="group border border-border p-6 transition-colors hover:border-foreground/20"
          >
            <h3
              class="mb-2 font-display text-lg font-semibold tracking-tight group-hover:text-primary"
            >
              {{ section.title }}
            </h3>
            <p class="font-sans text-sm leading-relaxed text-muted-foreground">
              {{ section.description }}
            </p>
            <span
              class="mt-4 inline-flex items-center gap-1.5 font-mono text-xs uppercase tracking-wider text-muted-foreground group-hover:text-foreground"
            >
              Read more
              <ArrowRight
                class="size-3 transition-transform group-hover:translate-x-0.5"
              />
            </span>
          </NuxtLink>
        </div>
      </div>
    </section>

    <!-- Footer -->
    <footer class="border-t border-border py-8">
      <div
        class="mx-auto flex max-w-6xl items-center justify-between px-6 text-muted-foreground"
      >
        <span class="font-mono text-xs">
          &copy; {{ new Date().getFullYear() }} Vinayak Kulkarni
        </span>
        <NuxtLink
          to="https://github.com/vinayakkulkarni/tileserver-rs"
          external
          class="font-mono text-xs hover:text-foreground"
        >
          GitHub
        </NuxtLink>
      </div>
    </footer>
    </div>
  </div>
</template>
