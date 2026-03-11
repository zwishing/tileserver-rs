<script setup lang="ts">
  import { Globe } from 'lucide-vue-next';
</script>

<template>
  <footer class="border-t border-border py-8">
    <div class="mx-auto max-w-6xl px-6">
      <div
class="
  flex flex-col items-center justify-between gap-4
  sm:flex-row
">
        <div class="flex items-center gap-2">
          <Globe class="size-4 text-primary" />
          <span
            class="font-display text-sm font-medium tracking-[0.15em] uppercase"
          >
            <span class="text-foreground">Tileserver</span>
            <span class="text-primary"> RS</span>
          </span>
        </div>
        <p class="text-sm text-muted-foreground">
          Built by
          <a
            href="https://vinayakkulkarni.dev"
            class="
              text-primary
              hover:underline
            "
          >
            Vinayak Kulkarni </a
          >. Open source under MIT License.
        </p>
      </div>
    </div>
  </footer>
</template>
