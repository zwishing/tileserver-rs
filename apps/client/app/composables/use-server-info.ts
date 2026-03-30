import type { PingResponse } from '~/types';

export function useServerInfo() {
  const { data, error } = useFetch<PingResponse>('/ping');

  const versionLabel = computed(() => {
    if (!data.value) return '';
    return `v${data.value.version}`;
  });

  return {
    ping: data,
    pingError: error,
    versionLabel,
  };
}
