import {
  Zap,
  Globe,
  RefreshCw,
  Database,
  MapIcon,
  HardDrive,
  Layers,
  Image,
  Server,
  Sparkles,
  Terminal,
  BarChart3,
} from 'lucide-vue-next';
import type {
  Feature,
  ApiEndpointGroup,
  PerformanceStat,
} from '~/types/marketing';

export function useMarketingPage() {
  const installCommand = 'brew install vinayakkulkarni/tap/tileserver-rs';
  const { copied, copy } = useClipboard({ copiedDuring: 2000 });

  async function copyToClipboard() {
    await copy(installCommand);
  }

  const features: Feature[] = [
    {
      icon: Zap,
      title: 'Blazing Fast',
      description:
        'Built in Rust for maximum performance. Serve tiles with sub-millisecond latency.',
    },
    {
      icon: Globe,
      title: 'PMTiles & MBTiles',
      description:
        'Native support for modern PMTiles and classic MBTiles tile archives with MVT and MLT format support.',
    },
    {
      icon: RefreshCw,
      title: 'MLT Transcoding',
      description:
        'On-the-fly MLT↔MVT transcoding. Serve next-gen MapLibre Tiles from existing MVT sources — up to 6x smaller tiles.',
    },
    {
      icon: Database,
      title: 'PostgreSQL / PostGIS',
      description:
        'Serve vector tiles directly from PostGIS tables with optimized spatial queries.',
    },
    {
      icon: MapIcon,
      title: 'Cloud Optimized GeoTIFF',
      description:
        'Serve raster tiles from COG files with on-the-fly reprojection and colormap support.',
    },
    {
      icon: HardDrive,
      title: 'PostgreSQL Out-DB Rasters',
      description:
        'Dynamic VRT/COG tile serving via PostGIS functions with query-based filtering.',
    },
    {
      icon: Layers,
      title: 'Vector & Raster',
      description:
        'Serve vector tiles directly or render them to raster on-the-fly.',
    },
    {
      icon: Image,
      title: 'Static Images',
      description:
        'Generate static map images like Mapbox Static API with native MapLibre rendering.',
    },
    {
      icon: Server,
      title: 'Self-Hosted',
      description:
        'Run on your own infrastructure. No vendor lock-in, no API keys required.',
    },
    {
      icon: Sparkles,
      title: 'Zero-Config Startup',
      description:
        'Point at a directory or file and start serving. Auto-detects PMTiles, MBTiles, styles, and fonts.',
    },
    {
      icon: Terminal,
      title: 'Hot-Reload',
      description:
        'Reload configuration without downtime via SIGHUP or admin API. Zero-request-drop with ArcSwap.',
    },
  ];

  const performanceStats: PerformanceStat[] = [
    {
      icon: Zap,
      value: 1409,
      label: 'PMTiles req/sec',
      detail: '10% faster than tileserver-gl',
    },
    {
      icon: Database,
      value: 3596,
      label: 'PostGIS req/sec',
      detail: 'Matches martin (PostGIS-bound)',
    },
    {
      icon: BarChart3,
      value: 13144,
      label: 'PostGIS req/sec at z14',
      detail: '7ms avg latency',
    },
    {
      icon: Layers,
      value: 93,
      label: 'MB/s throughput',
      detail: 'Consistent across zoom levels',
      prefix: '~',
    },
  ];

  const apiEndpoints: ApiEndpointGroup[] = [
    {
      title: 'Vector Tiles',
      endpoints: [
        { method: 'GET', path: '/data/{source}/{z}/{x}/{y}.pbf' },
        { method: 'GET', path: '/data/{source}/{z}/{x}/{y}.mlt' },
        { method: 'GET', path: '/data/{source}.json' },
      ],
    },
    {
      title: 'COG/Raster Tiles',
      endpoints: [
        { method: 'GET', path: '/data/{cog}/{z}/{x}/{y}.png' },
        { method: 'GET', path: '/data/{cog}/{z}/{x}/{y}.webp' },
      ],
    },
    {
      title: 'PostgreSQL Out-DB Rasters',
      endpoints: [
        {
          method: 'GET',
          path: '/data/{source}/{z}/{x}/{y}.png?satellite=...',
        },
      ],
    },
    {
      title: 'Style Rendering',
      endpoints: [
        { method: 'GET', path: '/styles/{style}/{z}/{x}/{y}.png' },
        { method: 'GET', path: '/styles/{style}/{z}/{x}/{y}@2x.png' },
      ],
    },
    {
      title: 'Static Images',
      endpoints: [
        {
          method: 'GET',
          path: '/styles/{style}/static/{lon},{lat},{zoom}/{w}x{h}.png',
        },
      ],
    },
  ];

  return {
    copied,
    installCommand,
    copyToClipboard,
    features,
    performanceStats,
    apiEndpoints,
  };
}
