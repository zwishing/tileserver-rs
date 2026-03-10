export function usePageSeo(options: {
  title: string;
  description: string;
  path: string;
  ogDescription?: string;
  ogImageAlt?: string;
  robots?: string;
}) {
  const config = useRuntimeConfig();
  const baseUrl = config.public.baseUrl || 'https://docs.tileserver.app';

  const canonicalUrl = `${baseUrl}${options.path}`;
  const ogDesc = options.ogDescription ?? options.description;
  const ogImageAlt = options.ogImageAlt ?? options.title;

  // Canonical link
  useHead({
    link: [{ rel: 'canonical', href: canonicalUrl }],
  });

  // Full SEO meta
  useSeoMeta({
    title: options.title,
    description: options.description,
    ...(options.robots ? { robots: options.robots } : {}),
    // Open Graph
    ogType: 'website',
    ogUrl: canonicalUrl,
    ogTitle: options.title,
    ogDescription: ogDesc,
    ogImageWidth: 1200,
    ogImageHeight: 630,
    ogImageAlt: ogImageAlt,
    ogSiteName: 'Tileserver RS Docs',
    // Twitter Card
    twitterCard: 'summary_large_image',
    twitterTitle: options.title,
    twitterDescription: ogDesc,
    twitterImageAlt: ogImageAlt,
  });
}
