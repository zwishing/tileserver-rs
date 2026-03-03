<script setup lang="ts">
  const configFeatures = [
    'Multiple tile sources (PMTiles, MBTiles, PostGIS, COG)',
    'Custom MapLibre GL styles',
    'In-memory tile caching with TTL',
    'Configurable CORS and connection pooling',
  ];
</script>

<template>
  <section class="border-t border-white/5 py-20">
    <div class="mx-auto max-w-6xl px-6">
      <div class="grid items-start gap-12 lg:grid-cols-2">
        <div>
          <p class="hud-label mb-4">Configuration</p>
          <h2
            class="mb-4 font-display text-3xl leading-[1.15] font-bold tracking-[-0.03em] sm:text-4xl"
          >
            Simple <span class="text-gradient">Setup</span>
          </h2>
          <p class="mb-8 font-sans text-lg/relaxed text-muted-foreground">
            Get started with a simple TOML configuration file. Define your tile
            sources, styles, and server settings in one place.
          </p>
          <ul class="space-y-3">
            <li
              v-for="item in configFeatures"
              :key="item"
              class="flex items-center gap-3 text-sm"
            >
              <span class="size-1.5 bg-primary"></span>
              <span class="text-foreground">{{ item }}</span>
            </li>
          </ul>
        </div>
        <div class="border border-white/10 bg-white/2">
          <div
            class="flex items-center gap-2 border-b border-white/10 px-4 py-2.5"
          >
            <span class="size-2.5 rounded-full bg-red-500/60"></span>
            <span class="size-2.5 rounded-full bg-yellow-500/60"></span>
            <span class="size-2.5 rounded-full bg-green-500/60"></span>
            <span class="ml-2 font-mono text-xs text-muted-foreground">
              config.toml
            </span>
          </div>
          <!-- eslint-disable vue/no-v-html -->
          <pre
            class="overflow-x-auto p-5 font-mono text-sm/relaxed"
          ><code><span class="token-comment"># Tile sources</span>
<span class="token-keyword">[[sources]]</span>
id = <span class="token-string">"openmaptiles"</span>
type = <span class="token-string">"pmtiles"</span>
path = <span class="token-string">"/data/tiles.pmtiles"</span>
serve_as = <span class="token-string">"mlt"</span>  <span class="token-comment"># MVT→MLT on the fly</span>

<span class="token-comment"># PostgreSQL / PostGIS</span>
<span class="token-keyword">[postgres]</span>
connection_string = <span class="token-string">"postgresql://user:pass@localhost/db"</span>

<span class="token-keyword">[[postgres.tables]]</span>
id = <span class="token-string">"buildings"</span>
table = <span class="token-string">"buildings"</span>
geometry_column = <span class="token-string">"geom"</span></code></pre>
        </div>
      </div>
    </div>
  </section>
</template>
