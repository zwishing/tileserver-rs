export default defineNuxtConfig({
  modules: [
    '@nuxt/content',
    '@nuxt/fonts',
    '@nuxt/eslint',
    '@nuxtjs/color-mode',
    '@vueuse/nuxt',
    'motion-v/nuxt',
    'shadcn-nuxt',
    [
      '@nuxtjs/plausible',
      {
        domain: 'docs.tileserver.app',
        apiHost: 'https://analytics.geoql.in',
        autoOutboundTracking: true,
      },
    ],
  ],

  devtools: { enabled: false },

  app: {
    head: {
      htmlAttrs: { lang: 'en' },
      title: 'Tileserver RS Docs - High-Performance Vector Tile Server',
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
        {
          name: 'description',
          content:
            'Documentation for tileserver-rs — a high-performance vector tile server built in Rust.',
        },
        {
          name: 'theme-color',
          content: '#111119',
          media: '(prefers-color-scheme: dark)',
        },
        {
          name: 'theme-color',
          content: '#ffffff',
          media: '(prefers-color-scheme: light)',
        },
      ],
      link: [{ rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' }],
    },
  },

  css: ['~/assets/css/main.css'],

  colorMode: {
    classSuffix: '',
    preference: 'dark',
    fallback: 'dark',
  },

  content: {
    build: {
      markdown: {
        highlight: {
          theme: {
            light: 'material-theme-lighter',
            default: 'material-theme',
            dark: 'material-theme-palenight',
          },
          langs: [
            'bash',
            'json',
            'js',
            'ts',
            'html',
            'css',
            'vue',
            'shell',
            'md',
            'yaml',
            'toml',
            'rust',
            'docker',
          ],
        },
      },
    },
    database: {
      type: 'd1',
      bindingName: 'DB',
    },
  },

  runtimeConfig: {
    public: {
      baseUrl: process.env.NUXT_PUBLIC_BASE_URL || 'https://docs.tileserver.app',
    },
  },

  future: {
    compatibilityVersion: 4,
  },

  compatibilityDate: '2025-07-18',

  nitro: {
    preset: 'cloudflare-pages',
    cloudflare: {
      nodeCompat: true,
    },
    rollupConfig: {
      output: {
        generatedCode: {
          constBindings: true,
        },
      },
    },
    replace: {
      'process.stdout': 'undefined',
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

  shadcn: {
    prefix: '',
    componentDir: './app/components/ui',
  },
});
