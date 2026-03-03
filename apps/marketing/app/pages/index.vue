<script setup lang="ts"></script>

<template>
  <div class="relative min-h-screen bg-background">
    <div class="bg-grid pointer-events-none fixed inset-0"></div>

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
