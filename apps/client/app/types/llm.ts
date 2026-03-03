/**
 * LLM Chat Types - Frontend-specific
 *
 * Types for browser-local LLM integration via WebLLM + TanStack AI Vue.
 * @see https://tanstack.com/ai/latest
 * @see https://webllm.mlc.ai/
 */

// Re-export core types from TanStack AI Vue
import type {
  UIMessage as TanStackUIMessage,
} from '@tanstack/ai-vue';
import type { MessagePart as TanStackMessagePart } from '@tanstack/ai';

export type {
  UseChatReturn,
} from '@tanstack/ai-vue';

// Re-export message part types from TanStack AI
export type {
  TextPart,
  ToolCallPart,
  ToolResultPart,
  MessagePart,
} from '@tanstack/ai';

/**
 * Readonly UIMessage type - matches what TanStack AI Vue returns from useChat.
 * The messages from useChat are DeepReadonly, so we need a compatible type for components.
 */
export interface ReadonlyUIMessage {
  readonly id: string;
  readonly role: 'system' | 'user' | 'assistant';
  readonly parts: readonly TanStackMessagePart[];
  readonly createdAt?: Date;
}

// Re-export original UIMessage for cases where mutable is needed
export type { TanStackUIMessage as UIMessage };

// ============================================================================
// WEBLLM TYPES
// ============================================================================

/**
 * WebLLM model configuration
 */
export interface LlmModelConfig {
  /** Model identifier for WebLLM (e.g., 'Qwen2.5-3B-Instruct-q4f16_1-MLC') */
  id: string;
  /** Human-readable display name */
  name: string;
  /** Approximate download size in GB */
  sizeGb: number;
}

/**
 * WebLLM engine loading progress
 */
export interface LlmLoadProgress {
  /** Progress text from WebLLM (e.g., 'Loading model...') */
  text: string;
  /** Progress value 0-1 */
  progress: number;
}

/**
 * WebLLM engine state
 */
export type LlmEngineStatus = 'idle' | 'loading' | 'ready' | 'error';

// ============================================================================
// MAP TOOLS
// ============================================================================

/**
 * Map tool definition for LLM tool calling
 */
export interface MapToolDefinition {
  name: string;
  description: string;
  parameters: Record<string, unknown>;
}

/**
 * Map tool call result
 */
export interface MapToolResult {
  toolCallId: string;
  toolName: string;
  result: unknown;
}

// ============================================================================
// PANEL TYPES
// ============================================================================

/**
 * Suggested prompts for the LLM chat (UI only)
 */
export interface SuggestedPrompt {
  title: string;
  prompt: string;
  icon: LlmIconName;
}

/**
 * Available icon names for suggested prompts
 */
export type LlmIconName = 'map' | 'layers' | 'search' | 'palette' | 'globe';

// ============================================================================
// COMPONENT PROPS TYPES
// ============================================================================

/**
 * Props for LlmPanel component
 */
export interface LlmPanelProps {
  open: boolean;
}

/**
 * Emits for LlmPanel component
 */
export interface LlmPanelEmits {
  (e: 'update:open', value: boolean): void;
}

/**
 * Props for LlmMessageList component
 */
export interface LlmMessageListProps {
  messages: readonly ReadonlyUIMessage[];
  isLoading: boolean;
}

/**
 * Props for LlmInput component
 */
export interface LlmInputProps {
  modelValue: string;
  isLoading: boolean;
  engineStatus: LlmEngineStatus;
  hasMessages: boolean;
}

/**
 * Emits for LlmInput component
 */
export interface LlmInputEmits {
  (e: 'update:modelValue', value: string): void;
  (e: 'submit' | 'stop'): void;
}
