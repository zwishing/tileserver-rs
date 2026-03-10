<script setup lang="ts">
  const { isDark } = useThemeToggle();

  usePageSeo({
    title: 'Tileserver RS - High-Performance Vector Tile Server',
    description:
      'High-performance vector tile server built in Rust with browser-local AI. Serve PMTiles and MBTiles with native MapLibre rendering — no API keys required.',
    path: '/',
  });

  const starColor = computed(() =>
    isDark.value ? '#a5b4fc' : '#6366f1',
  );

  const backgroundColor = computed(() =>
    isDark.value ? '#030014' : '#f8fafc',
  );
</script>

<template>
  <div class="relative min-h-dvh bg-background">
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
      <MarketingNavigation />

      <MarketingHeroSection />

      <FadeContent :duration="0.6">
        <MarketingFeaturesSection />
      </FadeContent>

      <FadeContent :duration="0.6">
        <MarketingAiSection />
      </FadeContent>

      <FadeContent :duration="0.6">
        <MarketingPerformanceSection />
      </FadeContent>

      <FadeContent :duration="0.6">
        <MarketingConfigSection />
      </FadeContent>

      <FadeContent :duration="0.6">
        <MarketingApiSection />
      </FadeContent>

      <FadeContent :duration="0.6">
        <MarketingCtaSection />
      </FadeContent>

      <MarketingFooter />
    </div>
  </div>
</template>
