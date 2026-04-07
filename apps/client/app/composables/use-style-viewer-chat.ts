import { useEventListener, useStorage } from '@vueuse/core';
import type { LlmPaletteMode } from '~/types/llm';

export function useStyleViewerChat() {
  const chatMode = useStorage<LlmPaletteMode>('tileserver-llm-mode', 'closed');
  const isChatVisible = computed(() => chatMode.value !== 'closed');

  function toggleChat() {
    chatMode.value = chatMode.value === 'closed' ? 'expanded' : 'closed';
  }

  useEventListener('keydown', (e: KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      toggleChat();
    }
    if (e.key === 'Escape' && chatMode.value === 'expanded') {
      chatMode.value = 'minimized';
    }
  });

  return { chatMode, isChatVisible, toggleChat };
}
