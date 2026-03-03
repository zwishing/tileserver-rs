<script setup lang="ts">
  import { Github, Globe, Moon, Sun } from 'lucide-vue-next';

  const { isDark, toggle } = useThemeToggle();
</script>

<template>
  <nav
    class="
      fixed top-0 z-50 w-full border-b border-border bg-background/80
      backdrop-blur-md
    "
  >
    <div class="mx-auto flex h-14 max-w-6xl items-center justify-between px-6">
      <NuxtLink to="/" class="flex items-center gap-2.5">
        <Globe class="size-5 text-primary" />
        <span
          class="font-display text-sm font-semibold tracking-[0.15em] uppercase"
        >
          <span class="text-foreground">Tileserver</span>
          <span class="text-primary"> RS</span>
        </span>
      </NuxtLink>
      <div class="flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          as="a"
          href="https://docs.tileserver.app"
          class="
            font-mono text-xs tracking-wider text-muted-foreground uppercase
            hover:text-foreground
          "
        >
          Docs
        </Button>
        <Button
          variant="ghost"
          size="sm"
          as="a"
          href="https://demo.tileserver.app"
          target="_blank"
          class="
            font-mono text-xs tracking-wider text-muted-foreground uppercase
            hover:text-foreground
          "
        >
          Demo
        </Button>
        <Button
          variant="outline"
          size="sm"
          as="a"
          href="https://github.com/vinayakkulkarni/tileserver-rs"
          class="
            gap-1.5 border-border bg-transparent font-mono text-xs
            tracking-wider uppercase
            hover:border-foreground/20 hover:bg-accent
          "
        >
          <Github class="size-3.5" />
          GitHub
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="size-9"
          aria-label="Toggle theme"
          @click="toggle"
        >
          <Sun v-if="isDark" class="size-4" />
          <Moon v-else class="size-4" />
        </Button>
      </div>
    </div>
  </nav>
</template>
