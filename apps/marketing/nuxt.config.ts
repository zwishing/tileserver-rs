export default defineNuxtConfig({
  modules: [
    'shadcn-nuxt',
    '@vueuse/nuxt',
    '@nuxt/eslint',
    '@nuxt/fonts',
    '@nuxtjs/color-mode',
    'motion-v/nuxt',
    [
      '@nuxtjs/plausible',
      {
        domain: 'tileserver.app',
        apiHost: 'https://analytics.geoql.in',
        autoOutboundTracking: true,
      },
    ],
  ],

  runtimeConfig: {
    public: {
      baseUrl: process.env.NUXT_PUBLIC_BASE_URL || 'https://tileserver.app',
    },
  },

  devtools: { enabled: false },

  app: {
    head: {
      htmlAttrs: { lang: 'en' },
      title: 'Tileserver RS - High-Performance Vector Tile Server',
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
        {
          name: 'description',
          content:
            'High-performance vector tile server built in Rust. Serve PMTiles and MBTiles with native MapLibre rendering.',
        },
        {
          name: 'keywords',
          content:
            'tileserver, vector tiles, pmtiles, mbtiles, maplibre, rust, gis, mapping, tile server',
        },
        { name: 'theme-color', content: '#111119' },
        {
          property: 'og:title',
          content: 'Tileserver RS - High-Performance Vector Tile Server',
        },
        {
          property: 'og:description',
          content:
            'High-performance vector tile server built in Rust. Serve PMTiles and MBTiles with native MapLibre rendering.',
        },
        { property: 'og:type', content: 'website' },
        { property: 'og:url', content: 'https://tileserver.app' },
        { name: 'twitter:card', content: 'summary_large_image' },
        {
          name: 'twitter:title',
          content: 'Tileserver RS - High-Performance Vector Tile Server',
        },
        {
          name: 'twitter:description',
          content:
            'High-performance vector tile server built in Rust. Serve PMTiles and MBTiles with native MapLibre rendering.',
        },
      ],
      link: [{ rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' }],
    },
  },

  css: ['~/assets/css/tailwind.css'],

  colorMode: {
    classSuffix: '',
    preference: 'dark',
    fallback: 'dark',
  },

  fonts: {
    families: [
      {
        name: 'General Sans',
        provider: 'fontshare',
        weights: [200, 300, 400, 500, 600, 700],
      },
      {
        name: 'Switzer',
        provider: 'fontshare',
        weights: [300, 400, 500, 600, 700],
      },
      { name: 'JetBrains Mono', provider: 'google' },
      { name: 'Source Serif 4', provider: 'google' },
    ],
  },

  future: {
    compatibilityVersion: 4,
  },

  experimental: {
    typedPages: true,
    viewTransition: true,
  },

  compatibilityDate: '2025-06-28',

  nitro: {
    preset: 'cloudflare-pages',
    prerender: {
      crawlLinks: true,
      routes: ['/'],
    },
  },

  vite: {
    ssr: {
      external: ['ogl'],
    },
  },

  typescript: {
    strict: true,
    typeCheck: false,
  },

  postcss: {
    plugins: {
      '@tailwindcss/postcss': {},
    },
  },

  shadcn: {
    prefix: '',
    componentDir: '@/components/ui',
  },
});
