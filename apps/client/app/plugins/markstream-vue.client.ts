/**
 * Markstream Vue Plugin (Client Only)
 *
 * Initializes markstream-vue for client-side markdown rendering.
 * The .client.ts suffix ensures Nuxt only loads this on the client.
 *
 * @see https://github.com/Simon-He95/markstream-vue
 */
import { disableKatex, disableMermaid } from 'markstream-vue';

export default defineNuxtPlugin(() => {
  // Disable KaTeX and Mermaid — we don't need math/diagrams in chat
  disableKatex();
  disableMermaid();
});
