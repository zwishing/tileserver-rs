/**
 * useChatHistory Composable
 *
 * Provides reactive chat history persistence via TanStack DB.
 * Messages and spatial results are stored in localStorage-backed
 * collections and reactively queried via useLiveQuery.
 *
 * @see https://tanstack.com/db/latest
 */

import { useLiveQuery, asc } from '@tanstack/vue-db';
import { chatCollection, spatialResultsCollection } from '~/lib/chat-db';
import type { ChatMessage, SpatialResult, StoredToolCall } from '~/types/llm';

/**
 * Reactive chat history with localStorage persistence.
 *
 * Provides:
 * - `messages` — reactive array of persisted chat messages (sorted by timestamp)
 * - `spatialResults` — reactive array of spatial query results
 * - `addMessage()` — persist a new chat message
 * - `addSpatialResult()` — persist a spatial query result
 * - `clearHistory()` — wipe all stored messages and results
 */
export function useChatHistory() {
  // Reactive query: all messages sorted by timestamp ascending
  const {
    data: messages,
    isLoading: messagesLoading,
  } = useLiveQuery((q) =>
    q.from({ chat: chatCollection })
      .orderBy(({ chat }) => asc(chat.timestamp))
      .select(({ chat }) => chat),
  );

  // Reactive query: all spatial results sorted by timestamp ascending
  const {
    data: spatialResults,
    isLoading: spatialResultsLoading,
  } = useLiveQuery((q) =>
    q.from({ results: spatialResultsCollection })
      .orderBy(({ results }) => asc(results.timestamp))
      .select(({ results }) => results),
  );

  /**
   * Persist a chat message to the collection.
   */
  function addMessage(msg: {
    id: string;
    role: 'user' | 'assistant';
    content: string;
    toolCalls?: StoredToolCall[];
  }): void {
    if (!import.meta.client) return;

    const chatMessage: ChatMessage = {
      id: msg.id,
      role: msg.role,
      content: msg.content,
      timestamp: Date.now(),
      ...(msg.toolCalls?.length ? { toolCalls: msg.toolCalls } : {}),
    };

    chatCollection.insert(chatMessage);
  }

  /**
   * Persist a spatial query result to the collection.
   */
  function addSpatialResult(result: Omit<SpatialResult, 'timestamp'>): void {
    if (!import.meta.client) return;

    spatialResultsCollection.insert({
      ...result,
      timestamp: Date.now(),
    });
  }

  /**
   * Clear all chat messages and spatial results from persistence.
   */
  function clearHistory(): void {
    if (!import.meta.client) return;

    // Delete all messages
    for (const msg of chatCollection.values()) {
      chatCollection.delete(msg.id);
    }

    // Delete all spatial results
    for (const result of spatialResultsCollection.values()) {
      spatialResultsCollection.delete(result.id);
    }
  }

  return {
    messages,
    spatialResults,
    messagesLoading,
    spatialResultsLoading,
    addMessage,
    addSpatialResult,
    clearHistory,
  };
}
