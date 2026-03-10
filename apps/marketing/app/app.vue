<script setup lang="ts">
  useHead({
    titleTemplate: (titleChunk) => {
      return titleChunk ? `${titleChunk} - Tileserver RS` : 'Tileserver RS';
    },
    script: [
      {
        type: 'application/ld+json',
        innerHTML: JSON.stringify({
          '@context': 'https://schema.org',
          '@type': 'SoftwareSourceCode',
          name: 'Tileserver RS',
          url: 'https://tileserver.app',
          description:
            'High-performance vector tile server built in Rust. Serve PMTiles and MBTiles with native MapLibre rendering.',
          codeRepository:
            'https://github.com/vinayakkulkarni/tileserver-rs',
          programmingLanguage: 'Rust',
          runtimePlatform: 'Docker',
          license: 'https://opensource.org/licenses/MIT',
          author: {
            '@type': 'Person',
            name: 'Vinayak Kulkarni',
            url: 'https://vinayakkulkarni.dev',
          },
        }),
      },
    ],
  });
</script>

<template>
  <NuxtPage />
</template>
