<script setup lang="ts">
  import { ArrowLeft, ArrowRight } from 'lucide-vue-next';
  import { useDocsNavigation } from '~/composables/useDocsNavigation';

  const route = useRoute();

  const path = computed(() =>
    route.path.endsWith('/') && route.path !== '/'
      ? route.path.slice(0, -1)
      : route.path,
  );

  const { data: page } = await useAsyncData(`page-${path.value}`, () =>
    queryCollection('content').path(path.value).first(),
  );

  if (!page.value) {
    throw createError({
      statusCode: 404,
      statusMessage: 'Page not found',
      fatal: true,
    });
  }

  const { sections } = useDocsNavigation();

  const allItems = computed(() => sections.flatMap((s) => s.children));
  const currentIndex = computed(() =>
    allItems.value.findIndex((item) => item.path === path.value),
  );
  const prevItem = computed(() =>
    currentIndex.value > 0 ? allItems.value[currentIndex.value - 1] : undefined,
  );
  const nextItem = computed(() =>
    currentIndex.value < allItems.value.length - 1
      ? allItems.value[currentIndex.value + 1]
      : undefined,
  );

  useSeoMeta({
    title: page.value?.title,
    description: page.value?.description,
    ogTitle: page.value?.title,
    ogDescription: page.value?.description,
  });
</script>

<template>
  <div
    v-if="page"
    class="mx-auto max-w-3xl px-6 py-12"
  >
    <!-- Breadcrumb -->
    <NuxtLink
      to="/"
      class="mb-8 inline-flex items-center gap-1.5 font-mono text-xs uppercase tracking-wider text-muted-foreground transition-colors hover:text-foreground"
    >
      <ArrowLeft class="size-3" />
      Documentation
    </NuxtLink>

    <!-- Content -->
    <article class="prose">
      <ContentRenderer :value="page" />
    </article>

    <!-- Prev / Next -->
    <nav
      v-if="prevItem || nextItem"
      class="mt-16 grid gap-4 border-t border-border pt-8 sm:grid-cols-2"
    >
      <NuxtLink
        v-if="prevItem"
        :to="prevItem.path"
        class="group border border-border p-4 transition-colors hover:border-foreground/20"
      >
        <span
          class="mb-1 block font-mono text-[10px] uppercase tracking-wider text-muted-foreground"
        >
          Previous
        </span>
        <span
          class="inline-flex items-center gap-1.5 font-display text-sm font-semibold tracking-tight group-hover:text-primary"
        >
          <ArrowLeft class="size-3" />
          {{ prevItem.title }}
        </span>
      </NuxtLink>
      <div v-else />
      <NuxtLink
        v-if="nextItem"
        :to="nextItem.path"
        class="group border border-border p-4 text-right transition-colors hover:border-foreground/20"
      >
        <span
          class="mb-1 block font-mono text-[10px] uppercase tracking-wider text-muted-foreground"
        >
          Next
        </span>
        <span
          class="inline-flex items-center gap-1.5 font-display text-sm font-semibold tracking-tight group-hover:text-primary"
        >
          {{ nextItem.title }}
          <ArrowRight class="size-3" />
        </span>
      </NuxtLink>
    </nav>
  </div>
</template>
