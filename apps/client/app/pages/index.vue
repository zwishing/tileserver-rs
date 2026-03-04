<script setup lang="ts">
  const {
    isDark,
    toggleColorMode,
    isLoadingData,
    isLoadingStyles,
    hasStyles,
    hasData,
    searchQuery,
    filteredStyles,
    filteredDataSources,
    expandedStyleXyz,
    expandedDataXyz,
    toggleStyleXyz,
    toggleDataXyz,
    copiedUrl,
    copyUrl,
    stylesOpen,
    dataOpen,
    baseUrl,
    versionLabel,
  } = useHomePage();
</script>

<template>
  <div class="flex min-h-dvh flex-col bg-background">
    <HomeHero :is-dark="isDark" @toggle-theme="toggleColorMode" />

    <main class="w-full flex-1 space-y-4 p-4">
      <HomeSearchBar v-model="searchQuery" />

      <HomeStyleList
        :styles="filteredStyles"
        :is-loading="isLoadingStyles"
        :has-styles="hasStyles"
        :is-open="stylesOpen"
        :search-query="searchQuery"
        :base-url="baseUrl"
        :expanded-xyz="expandedStyleXyz"
        :copied-url="copiedUrl"
        @update:is-open="stylesOpen = $event"
        @toggle-xyz="toggleStyleXyz"
        @copy-url="copyUrl"
      />

      <HomeDataList
        :sources="filteredDataSources"
        :is-loading="isLoadingData"
        :has-data="hasData"
        :is-open="dataOpen"
        :search-query="searchQuery"
        :base-url="baseUrl"
        :expanded-xyz="expandedDataXyz"
        :copied-url="copiedUrl"
        @update:is-open="dataOpen = $event"
        @toggle-xyz="toggleDataXyz"
        @copy-url="copyUrl"
      />

      <HomeApiLink />
    </main>

    <HomeFooter :version-label="versionLabel" />
  </div>
</template>
