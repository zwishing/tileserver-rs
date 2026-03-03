/**
 * useLlmEngine Composable
 *
 * Manages the WebLLM engine lifecycle: initialization, model loading,
 * progress tracking, and cleanup. The engine runs entirely in-browser
 * via WebGPU — no server required.
 *
 * @see https://webllm.mlc.ai/
 */

import { CreateMLCEngine } from '@mlc-ai/web-llm';
import type { MLCEngine } from '@mlc-ai/web-llm';
import type { LlmEngineStatus, LlmLoadProgress, LlmModelConfig } from '~/types/llm';

/** Available models for the chat */
const AVAILABLE_MODELS: LlmModelConfig[] = [
  {
    id: 'Qwen2.5-3B-Instruct-q4f16_1-MLC',
    name: 'Qwen 2.5 3B (recommended)',
    sizeGb: 2.0,
  },
  {
    id: 'Qwen2.5-1.5B-Instruct-q4f16_1-MLC',
    name: 'Qwen 2.5 1.5B (lightweight)',
    sizeGb: 1.0,
  },
  {
    id: 'Llama-3.2-3B-Instruct-q4f16_1-MLC',
    name: 'Llama 3.2 3B',
    sizeGb: 2.0,
  },
];

const DEFAULT_MODEL = AVAILABLE_MODELS[0];

/**
 * Shared engine state — singleton across the app.
 * The engine persists across component mounts so the model
 * isn't re-downloaded when toggling the chat panel.
 */
const engine = shallowRef<MLCEngine | null>(null);
const status = ref<LlmEngineStatus>('idle');
const loadProgress = ref<LlmLoadProgress>({ text: '', progress: 0 });
const errorMessage = ref<string | null>(null);
const selectedModel = ref<LlmModelConfig>(DEFAULT_MODEL);

/**
 * Composable for managing WebLLM engine lifecycle.
 *
 * @example
 * ```ts
 * const { engine, status, loadProgress, initEngine } = useLlmEngine();
 * await initEngine(); // Downloads + compiles model (~30s first time)
 * ```
 */
export function useLlmEngine() {
  /**
   * Check if WebGPU is available in this browser
   */
  function isWebGpuSupported(): boolean {
    return typeof navigator !== 'undefined' && 'gpu' in navigator;
  }

  /**
   * Initialize the WebLLM engine with the selected model.
   * Downloads and compiles the model on first run (~30s).
   * Subsequent loads use cached model from IndexedDB (~2-5s).
   */
  async function initEngine(modelConfig?: LlmModelConfig): Promise<void> {
    if (status.value === 'loading') return;
    if (engine.value && status.value === 'ready') return;

    if (!isWebGpuSupported()) {
      status.value = 'error';
      errorMessage.value = 'WebGPU is not supported in this browser. Try Chrome 113+ or Edge 113+.';
      return;
    }

    const model = modelConfig ?? selectedModel.value;
    selectedModel.value = model;
    status.value = 'loading';
    errorMessage.value = null;
    loadProgress.value = { text: 'Initializing...', progress: 0 };

    try {
      const mlcEngine = await CreateMLCEngine(model.id, {
        initProgressCallback: (progress) => {
          loadProgress.value = {
            text: progress.text,
            progress: progress.progress,
          };
        },
      });

      engine.value = mlcEngine;
      status.value = 'ready';
      loadProgress.value = { text: 'Ready', progress: 1 };
    } catch (err) {
      status.value = 'error';
      errorMessage.value = err instanceof Error ? err.message : 'Failed to load model';
      console.error('[LLM Engine] Failed to initialize:', err);
    }
  }

  /**
   * Reset the engine (unload model, free memory)
   */
  async function resetEngine(): Promise<void> {
    if (engine.value) {
      try {
        await engine.value.resetChat();
      } catch {
        // Ignore reset errors
      }
      engine.value = null;
    }
    status.value = 'idle';
    loadProgress.value = { text: '', progress: 0 };
    errorMessage.value = null;
  }

  return {
    engine: engine as Readonly<ShallowRef<MLCEngine | null>>,
    status: readonly(status),
    loadProgress: readonly(loadProgress),
    errorMessage: readonly(errorMessage),
    selectedModel: readonly(selectedModel),
    availableModels: AVAILABLE_MODELS,
    isWebGpuSupported,
    initEngine,
    resetEngine,
  };
}
