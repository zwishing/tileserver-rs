<script setup lang="ts">
  const { isDark } = useThemeToggle();

  const squareBorderColor = computed(() =>
    isDark.value ? 'oklch(1 0 0 / 0.06)' : 'oklch(0 0 0 / 0.06)',
  );

  const squareHoverColor = computed(() =>
    isDark.value ? 'oklch(1 0 0 / 0.03)' : 'oklch(0 0 0 / 0.04)',
  );
</script>

<template>
  <div class="relative min-h-dvh bg-background">
    <div class="pointer-events-none fixed inset-0">
      <Squares
        :speed="0.3"
        direction="diagonal"
        :border-color="squareBorderColor"
        :hover-fill-color="squareHoverColor"
        :square-size="40"
      />
    </div>

    <div class="relative z-10">
      <MarketingNavigation />

      <MarketingHeroSection />

      <FadeContent :duration="0.6">
        <MarketingFeaturesSection />
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
