<script setup lang="ts">
  import {
    ArrowRight,
    ExternalLink,
    BotMessageSquare,
    User,
  } from 'lucide-vue-next';

  const { aiBenefits, aiChatExample } = useMarketingPage();
</script>

<template>
  <section
    data-label="AI Assistant"
    class="section-full relative border-b border-border"
  >
    <div
      class="geometric-grid-bg pointer-events-none absolute inset-0 opacity-30"
      aria-hidden="true"
    />

    <div class="relative px-6 py-16 md:px-12 lg:px-20">
      <div class="mb-16">
        <p class="hud-label mb-4">AI Assistant</p>
        <h2
          class="
            mb-4 font-display text-3xl leading-[1.15] font-bold
            tracking-[-0.03em]
            sm:text-4xl
          "
        >
          AI Without the <span class="text-gradient">Landlord</span>
        </h2>
        <p
          class="
            mx-auto max-w-2xl font-sans text-lg/relaxed text-muted-foreground
          "
        >
          Your maps, your GPU, your data. The built-in AI assistant runs
          entirely in your browser &mdash; no API keys, no cloud inference, no
          per-token billing. Just open the chat and talk to your maps.
        </p>
      </div>

      <div
        class="
          grid gap-6
          sm:grid-cols-2
          lg:grid-cols-4
        "
      >
        <SpotlightCard
          v-for="benefit in aiBenefits"
          :key="benefit.title"
          :border-radius="0"
          spotlight-color="rgba(139, 92, 246, 0.08)"
        >
          <SpotlightCardHeader>
            <div
              class="
                mb-3 inline-flex size-10 items-center justify-center border
                border-primary/20 bg-primary/5 text-primary
              "
            >
              <component :is="benefit.icon" class="size-5" />
            </div>
            <SpotlightCardTitle class="text-base">
              {{ benefit.title }}
            </SpotlightCardTitle>
          </SpotlightCardHeader>
          <SpotlightCardContent>
            <SpotlightCardDescription>
              {{ benefit.description }}
            </SpotlightCardDescription>
          </SpotlightCardContent>
        </SpotlightCard>
      </div>

      <div class="mt-12 grid items-start gap-12 lg:grid-cols-2">
        <div class="overflow-hidden border border-border bg-muted/30">
          <div
            class="
              flex items-center gap-2 border-b border-border px-4 py-2.5
            "
          >
            <BotMessageSquare class="size-3.5 text-primary" />
            <span class="font-mono text-xs text-muted-foreground">
              AI Chat — browser-local, WebGPU
            </span>
          </div>
          <div class="space-y-4 p-5">
            <div
              v-for="(msg, i) in aiChatExample"
              :key="i"
              class="flex gap-3"
            >
              <div
                class="
                  mt-0.5 flex size-6 shrink-0 items-center justify-center
                  border
                "
                :class="
                  msg.role === 'user'
                    ? 'border-border bg-muted text-muted-foreground'
                    : 'border-primary/30 bg-primary/10 text-primary'
                "
              >
                <User v-if="msg.role === 'user'" class="size-3" />
                <BotMessageSquare v-else class="size-3" />
              </div>
              <div class="min-w-0 flex-1">
                <p
                  class="mb-0.5 font-mono text-[10px] tracking-wider uppercase"
                  :class="
                    msg.role === 'user'
                      ? 'text-muted-foreground'
                      : 'text-primary'
                  "
                >
                  {{ msg.role === 'user' ? 'You' : 'AI' }}
                </p>
                <p
                  class="whitespace-pre-line font-mono text-sm"
                  :class="
                    msg.role === 'user'
                      ? 'text-foreground'
                      : 'text-muted-foreground'
                  "
                >
                  {{ msg.text }}
                </p>
              </div>
            </div>
          </div>
        </div>

        <div class="flex flex-col justify-center">
          <h3
            class="
              mb-4 font-display text-2xl leading-[1.15] font-bold
              tracking-[-0.02em]
            "
          >
            No tokens. No telemetry. No landlords.
          </h3>
          <p class="mb-4 font-sans text-base/relaxed text-muted-foreground">
            The AI runs a quantized LLM directly on your GPU via WebGPU &mdash;
            the same technology that powers browser gaming. First load downloads
            the model (~2 GB), then it&rsquo;s cached in your browser forever.
          </p>
          <ul class="mb-8 space-y-2">
            <li class="flex items-center gap-3 text-sm">
              <span class="size-1.5 bg-emerald-400"></span>
              <span class="text-foreground">
                Works offline after first model download
              </span>
            </li>
            <li class="flex items-center gap-3 text-sm">
              <span class="size-1.5 bg-emerald-400"></span>
              <span class="text-foreground">
                10+ map tools: navigate, filter, style, query
              </span>
            </li>
            <li class="flex items-center gap-3 text-sm">
              <span class="size-1.5 bg-emerald-400"></span>
              <span class="text-foreground">
                Spatial queries against your tile data
              </span>
            </li>
            <li class="flex items-center gap-3 text-sm">
              <span class="size-1.5 bg-emerald-400"></span>
              <span class="text-foreground">
                Chat history persisted locally in IndexedDB
              </span>
            </li>
          </ul>
          <div class="flex gap-3">
            <Button
              size="lg"
              as="a"
              href="https://demo.tileserver.app"
              target="_blank"
              class="
                group gap-2 bg-primary px-6 text-primary-foreground
                hover:bg-primary/90
              "
            >
              <ExternalLink class="size-4" />
              Try It Live
              <ArrowRight
                class="
                  size-4 transition-transform
                  group-hover:translate-x-0.5
                "
              />
            </Button>
            <Button
              variant="outline"
              size="lg"
              as="a"
              href="https://docs.tileserver.app/guides/ai-chat"
              class="
                gap-2 border-border bg-transparent
                hover:border-foreground/20 hover:bg-accent
              "
            >
              Learn More
            </Button>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
