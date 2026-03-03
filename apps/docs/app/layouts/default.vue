<script setup lang="ts">
  import { Github, Globe, Menu, Moon, Sun, X } from 'lucide-vue-next';

  const {
    sections,
    sidebarOpen,
    toggleSidebar,
    closeSidebar,
    isActive,
    isInSection,
  } = useDocsNavigation();

  const { isDark, toggle: toggleTheme } = useThemeToggle();

  const route = useRoute();

  watch(
    () => route.path,
    () => closeSidebar(),
  );
</script>

<template>
  <div class="min-h-dvh bg-background">
    <!-- Top nav -->
    <nav
      class="fixed top-0 z-50 w-full border-b border-border bg-background/80 backdrop-blur-md"
    >
      <div class="flex h-14 items-center justify-between px-6">
        <div class="flex items-center gap-4">
          <button
            class="text-muted-foreground hover:text-foreground lg:hidden"
            @click="toggleSidebar()"
          >
            <Menu
              v-if="!sidebarOpen"
              class="size-5"
            />
            <X
              v-else
              class="size-5"
            />
          </button>
          <NuxtLink
            to="/"
            class="flex items-center gap-2.5"
          >
            <Globe class="size-5 text-primary" />
            <span
              class="font-display text-sm font-semibold uppercase tracking-[0.15em]"
            >
              <span class="text-foreground">Tileserver</span>
              <span class="text-primary"> RS</span>
            </span>
            <span
              class="border border-border bg-muted px-1.5 py-0.5 font-mono text-[10px] uppercase tracking-wider text-muted-foreground"
            >
              docs
            </span>
          </NuxtLink>
        </div>
        <div class="flex items-center gap-1">
          <NuxtLink
            to="https://tileserver.app"
            external
            class="px-3 py-1.5 font-mono text-xs uppercase tracking-wider text-muted-foreground hover:text-foreground"
          >
            Home
          </NuxtLink>
          <NuxtLink
            to="https://github.com/vinayakkulkarni/tileserver-rs"
            external
            class="flex items-center gap-1.5 border border-border bg-transparent px-3 py-1.5 font-mono text-xs uppercase tracking-wider text-muted-foreground hover:border-foreground/20 hover:text-foreground"
          >
            <Github class="size-3.5" />
            GitHub
          </NuxtLink>
          <button
            class="flex size-9 items-center justify-center text-muted-foreground transition-colors hover:text-foreground"
            aria-label="Toggle theme"
            @click="toggleTheme"
          >
            <Sun
              v-if="isDark"
              class="size-4"
            />
            <Moon
              v-else
              class="size-4"
            />
          </button>
        </div>
      </div>
    </nav>

    <!-- Sidebar overlay -->
    <div
      v-if="sidebarOpen"
      class="fixed inset-0 z-30 bg-black/50 lg:hidden"
      @click="closeSidebar()"
    />

    <!-- Sidebar -->
    <aside
      :class="[
        'fixed top-14 bottom-0 z-40 w-64 overflow-y-auto border-r border-border bg-background px-4 py-6',
        'transition-transform lg:translate-x-0',
        sidebarOpen ? 'translate-x-0' : '-translate-x-full',
      ]"
    >
      <nav class="space-y-6">
        <div
          v-for="section in sections"
          :key="section.path"
        >
          <p
            class="mb-2 font-mono text-xs font-medium uppercase tracking-[0.15em] text-muted-foreground"
          >
            {{ section.title }}
          </p>
          <ul class="space-y-0.5">
            <li
              v-for="item in section.children"
              :key="item.path"
            >
              <NuxtLink
                :to="item.path"
                :class="[
                  'block py-1.5 pl-3 text-sm transition-colors',
                  'border-l-2',
                  isActive(item.path)
                    ? 'border-primary text-foreground font-medium'
                    : isInSection(section.path)
                      ? 'border-border text-muted-foreground hover:text-foreground hover:border-foreground/30'
                      : 'border-transparent text-muted-foreground hover:text-foreground hover:border-foreground/30',
                ]"
              >
                {{ item.title }}
              </NuxtLink>
            </li>
          </ul>
        </div>
      </nav>
    </aside>

    <!-- Main content -->
    <main class="pt-14 lg:pl-64">
      <slot />
    </main>
  </div>
</template>
