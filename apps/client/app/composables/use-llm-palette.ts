import type { Map as MaplibreMap } from 'maplibre-gl';
import type { OverlayLayer } from '~/types/file-upload';

export function useLlmPalette(
  openRef: Ref<boolean>,
  mapRef: Ref<MaplibreMap | null>,
  overlaysRef: Ref<readonly OverlayLayer[]>,
) {
  const panelData = useLlmPanel(mapRef, overlaysRef);
  const { scrollAnchor } = panelData;

  const panelRef = ref<HTMLElement | null>(null);

  watch(openRef, (isOpen) => {
    if (!isOpen) return;
    nextTick(() => {
      panelRef.value?.querySelector<HTMLInputElement>('input')?.focus();
      nextTick(() => scrollAnchor.value?.scrollIntoView({ block: 'end' }));
    });
  });

  return {
    ...panelData,
    panelRef,
  };
}
