import type { PingResponse } from '~/types';

export function useServerInfo() {
  const { data } = useFetch<PingResponse>('/ping');

  const versionLabel = computed(() => {
    if (!data.value) return '';
    return `v${data.value.version}`;
  });

  return {
    ping: data,
    versionLabel,
  };
}
