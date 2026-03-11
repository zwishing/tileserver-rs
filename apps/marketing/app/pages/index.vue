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

  const activeLabel = ref('Tileserver RS');

  onMounted(() => {
    const sections = document.querySelectorAll<HTMLElement>(
      'section[data-label]',
    );
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            activeLabel.value = entry.target.dataset.label ?? '';
          }
        }
      },
      { rootMargin: '-40% 0px -40% 0px' },
    );
    sections.forEach((s) => observer.observe(s));
    onUnmounted(() => observer.disconnect());
  });
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

      <!-- Fixed left gutter -->
      <div
        class="
          fixed top-[72px] left-0 z-20 flex w-12 items-center justify-center
          border-r border-border bg-background
          lg:top-[80px] lg:w-20
        "
      >
        <Transition
          name="fade"
          mode="out-in"
        >
          <span
            :key="activeLabel"
            class="
              vertical-text font-mono text-[10px] tracking-[0.3em]
              text-muted-foreground uppercase
            "
          >
            {{ activeLabel }}
          </span>
        </Transition>
      </div>

      <!-- Content with left margin -->
      <div
class="
  mt-[72px] ml-12
  lg:mt-[80px] lg:ml-20
">
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
  </div>
</template>