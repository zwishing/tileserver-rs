<script setup lang="ts">
  const {
    isDark,
    toggleColorMode,
    versionLabel,
    searchQuery,
    filteredStyles,
    filteredDataSources,
    isLoadingData,
    isLoadingStyles,
    hasStyles,
    hasData,
    expandedStyleXyz,
    expandedDataXyz,
    toggleStyleXyz,
    toggleDataXyz,
    copiedUrl,
    copyUrl,
    stylesOpen,
    dataOpen,
    baseUrl,
  } = useHomePage();
</script>

<template>
  <div class="flex min-h-dvh flex-col bg-background">
    <HomeHeader :is-dark="isDark" @toggle-theme="toggleColorMode" />

    <main class="w-full flex-1 space-y-4 px-6 py-4">
      <HomeSearchBar v-model="searchQuery" />

      <HomeStylesSection
        v-model:open="stylesOpen"
        :styles="filteredStyles"
        :is-loading="isLoadingStyles"
        :has-styles="hasStyles"
        :search-query="searchQuery"
        :expanded-xyz="expandedStyleXyz"
        :copied-url="copiedUrl"
        :base-url="baseUrl"
        @toggle-xyz="toggleStyleXyz"
        @copy-url="copyUrl"
      />

      <HomeDataSection
        v-model:open="dataOpen"
        :sources="filteredDataSources"
        :is-loading="isLoadingData"
        :has-data="hasData"
        :search-query="searchQuery"
        :expanded-xyz="expandedDataXyz"
        :copied-url="copiedUrl"
        :base-url="baseUrl"
        @toggle-xyz="toggleDataXyz"
        @copy-url="copyUrl"
      />

      <HomeApiLink />
    </main>

    <HomeFooter :version-label="versionLabel" />
  </div>
</template>
