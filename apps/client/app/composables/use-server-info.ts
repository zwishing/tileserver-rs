import type { PingResponse } from '~/types';

export function useServerInfo() {
  const { data } = useFetch<PingResponse>('/ping');

  const versionLabel = computed(() => {
    if (!data.value) return '';
    const v = data.value.version;
    const hash = data.value.git_hash;
    return hash ? `v${v} (${hash})` : `v${v}`;
  });

  return {
    ping: data,
    versionLabel,
  };
}
