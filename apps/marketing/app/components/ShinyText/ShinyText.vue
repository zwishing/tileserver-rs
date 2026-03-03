<script setup lang="ts">
  import { computed } from 'vue';
  import { cn } from '~/lib/utils';

  const props = withDefaults(
    defineProps<{
      text: string;
      disabled?: boolean;
      speed?: number;
      class?: string;
    }>(),
    {
      disabled: false,
      speed: 5,
      class: '',
    },
  );

  const animationDuration = computed(() => `${props.speed}s`);
</script>

<template>
  <span
    :class="
      cn(
        `
          shiny-text inline-block bg-size-[200%_100%] bg-clip-text
          text-transparent
        `,
        !disabled && 'animate-[shiny-sweep_var(--shiny-speed)_linear_infinite]',
        props.class,
      )
    "
    :style="{
      '--shiny-speed': animationDuration,
    }"
  >
    {{ text }}
  </span>
</template>

<style scoped>
  @keyframes shiny-sweep {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  .shiny-text {
    animation: shiny-sweep var(--shiny-speed, 5s) linear infinite;
  }

  :root {
    .shiny-text {
      background-image: linear-gradient(
        120deg,
        oklch(0.541 0.281 293.009) 40%,
        oklch(0.702 0.183 293.541) 50%,
        oklch(0.541 0.281 293.009) 60%
      );
    }
  }

  .dark .shiny-text {
    background-image: linear-gradient(
      120deg,
      oklch(0.541 0.281 293.009) 40%,
      oklch(0.902 0.153 293.541) 50%,
      oklch(0.541 0.281 293.009) 60%
    );
  }
</style>
