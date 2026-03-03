/**
 * useLlmChat Composable
 *
 * Bridges WebLLM (browser-local LLM) with TanStack AI Vue's useChat.
 * Uses the `stream()` adapter to convert WebLLM's OpenAI-compatible
 * streaming output to AG-UI protocol events.
 *
 * @see https://tanstack.com/ai/latest
 * @see https://webllm.mlc.ai/
 */

import { useChat, stream } from '@tanstack/ai-vue';
import type { UseChatReturn, UIMessage } from '@tanstack/ai-vue';
import type { Map as MaplibreMap } from 'maplibre-gl';

/**
 * Map tool definitions for OpenAI-compatible tool calling.
 * These let the LLM interact with the MapLibre GL map instance.
 */
const MAP_TOOLS = [
  {
    type: 'function' as const,
    function: {
      name: 'fly_to',
      description: 'Animate the map camera to a specific location',
      parameters: {
        type: 'object',
        properties: {
          lng: { type: 'number', description: 'Longitude (-180 to 180)' },
          lat: { type: 'number', description: 'Latitude (-90 to 90)' },
          zoom: { type: 'number', description: 'Zoom level (0-22)', default: 12 },
          bearing: { type: 'number', description: 'Bearing in degrees', default: 0 },
          pitch: { type: 'number', description: 'Pitch in degrees (0-85)', default: 0 },
        },
        required: ['lng', 'lat'],
      },
    },
  },
  {
    type: 'function' as const,
    function: {
      name: 'fit_bounds',
      description: 'Fit the map camera to a bounding box',
      parameters: {
        type: 'object',
        properties: {
          west: { type: 'number', description: 'West longitude' },
          south: { type: 'number', description: 'South latitude' },
          east: { type: 'number', description: 'East longitude' },
          north: { type: 'number', description: 'North latitude' },
          padding: { type: 'number', description: 'Padding in pixels', default: 50 },
        },
        required: ['west', 'south', 'east', 'north'],
      },
    },
  },
  {
    type: 'function' as const,
    function: {
      name: 'get_map_state',
      description: 'Get the current map center, zoom, bearing, pitch, and visible layers',
      parameters: {
        type: 'object',
        properties: {},
      },
    },
  },
];

/**
 * Model prefixes known to support tool calling in WebLLM.
 * WebLLM throws "not supported for ChatCompletionRequest.tools" for others.
 * @see https://webllm.mlc.ai/
 */
const TOOL_CAPABLE_PREFIXES = [
  'Hermes-2-Pro-',
  'Hermes-3-Llama-',
];

function modelSupportsTools(modelId: string): boolean {
  return TOOL_CAPABLE_PREFIXES.some((prefix) => modelId.startsWith(prefix));
}

/**
 * System prompt for tool-capable models (Hermes)
 */
const SYSTEM_PROMPT_WITH_TOOLS = `You are a helpful map assistant embedded in tileserver-rs, a vector tile server.
You can help users explore the map by flying to locations, adjusting the view, and answering questions about geography.

When users ask to see a place, use fly_to with coordinates.
When users ask to see a region, use fit_bounds with a bounding box.
Use get_map_state to understand what the user is currently looking at.

Keep responses concise and helpful. You're a map expert.`;

/**
 * System prompt for non-tool models (Qwen) — uses structured JSON action blocks.
 * The model emits [MAP_ACTION]{...}[/MAP_ACTION] which we parse and execute client-side.
 */
const SYSTEM_PROMPT_NO_TOOLS = `You are a helpful map assistant embedded in tileserver-rs, a vector tile server.
You can help users explore the map by flying to locations, adjusting the view, and answering questions about geography.

IMPORTANT: You can control the map directly. When a user asks to navigate somewhere, you MUST include a map action block in your response using this exact format:

[MAP_ACTION]{"action":"fly_to","lng":<longitude>,"lat":<latitude>,"zoom":<zoom_level>}[/MAP_ACTION]

Available actions:
- fly_to: Navigate to a location. Required: lng, lat. Optional: zoom (default 12), bearing (default 0), pitch (default 0).
  Example: [MAP_ACTION]{"action":"fly_to","lng":2.3522,"lat":48.8566,"zoom":13}[/MAP_ACTION]
- fit_bounds: Fit map to a bounding box. Required: west, south, east, north. Optional: padding (default 50).
  Example: [MAP_ACTION]{"action":"fit_bounds","west":-5.0,"south":42.0,"east":9.5,"north":51.0}[/MAP_ACTION]

ALWAYS include the [MAP_ACTION] block when the user asks to go somewhere, fly to a place, or show a location. Put it at the END of your response after your text explanation.
Keep responses concise and helpful. You're a map expert.`;

/**
 * Extract text content from a UIMessage
 */
function extractText(message: UIMessage): string {
  return message.parts
    .filter((p) => p.type === 'text')
    .map((p) => (p as { type: 'text'; content: string }).content)
    .join('');
}

/**
 * Parse [MAP_ACTION]{...}[/MAP_ACTION] blocks from LLM response text.
 * Returns parsed action objects and the cleaned text without action blocks.
 */
function parseMapActions(text: string): { actions: Array<{ action: string } & Record<string, unknown>>; cleanText: string } {
  const regex = /\[MAP_ACTION\](\{[\s\S]*?\})\[\/MAP_ACTION\]/g;
  const actions: Array<{ action: string } & Record<string, unknown>> = [];
  let match: RegExpExecArray | null;

  while ((match = regex.exec(text)) !== null) {
    try {
      const parsed = JSON.parse(match[1]) as { action: string } & Record<string, unknown>;
      if (parsed.action) {
        actions.push(parsed);
      }
    } catch {
      // Invalid JSON in action block — skip
    }
  }

  // Remove action blocks from displayed text
  const cleanText = text.replace(/\[MAP_ACTION\]\{[\s\S]*?\}\[\/MAP_ACTION\]/g, '').trim();

  return { actions, cleanText };
}

/**
 * Execute a map tool call against the MapLibre instance
 */
function executeMapTool(
  toolName: string,
  args: Record<string, unknown>,
  map: MaplibreMap | null,
): unknown {
  if (!map) return { error: 'Map not available' };

  switch (toolName) {
    case 'fly_to': {
      const { lng, lat, zoom = 12, bearing = 0, pitch = 0 } = args as {
        lng: number;
        lat: number;
        zoom?: number;
        bearing?: number;
        pitch?: number;
      };
      map.flyTo({ center: [lng, lat], zoom, bearing, pitch, duration: 2000 });
      return { success: true, message: `Flying to [${lng}, ${lat}] at zoom ${zoom}` };
    }
    case 'fit_bounds': {
      const { west, south, east, north, padding = 50 } = args as {
        west: number;
        south: number;
        east: number;
        north: number;
        padding?: number;
      };
      map.fitBounds([[west, south], [east, north]], { padding, duration: 2000 });
      return { success: true, message: `Fitting to bounds [${west},${south},${east},${north}]` };
    }
    case 'get_map_state': {
      const center = map.getCenter();
      const zoom = map.getZoom();
      const bearing = map.getBearing();
      const pitch = map.getPitch();
      const layers = map.getStyle()?.layers?.map((l) => l.id).slice(0, 20) ?? [];
      return {
        center: { lng: center.lng, lat: center.lat },
        zoom: Math.round(zoom * 100) / 100,
        bearing: Math.round(bearing),
        pitch: Math.round(pitch),
        visibleLayers: layers,
      };
    }
    default:
      return { error: `Unknown tool: ${toolName}` };
  }
}

/**
 * Composable for LLM chat with map tool integration.
 *
 * Creates a TanStack AI `useChat` instance connected to WebLLM
 * via the `stream()` adapter. Supports map tool calling.
 *
 * @param mapRef - Ref to the MapLibre GL map instance
 */
export function useLlmChat(mapRef: Ref<MaplibreMap | null>): UseChatReturn {
  const { engine, status, selectedModel } = useLlmEngine();

  /**
   * Create the connection adapter that bridges WebLLM → AG-UI events.
   * The `stream()` helper from @tanstack/ai-vue converts an async generator
   * into a ConnectionAdapter compatible with useChat.
   */
  const connection = stream(async function* (messages: UIMessage[]) {
    const currentEngine = engine.value;
    if (!currentEngine) {
      throw new Error('LLM engine not initialized. Please wait for model to load.');
    }

    const runId = crypto.randomUUID();
    const messageId = crypto.randomUUID();

    yield { type: 'RUN_STARTED' as const, runId, timestamp: Date.now() };
    yield { type: 'TEXT_MESSAGE_START' as const, messageId, role: 'assistant' as const, timestamp: Date.now() };

    // Select system prompt based on tool support
    const useTools = modelSupportsTools(selectedModel.value.id);
    const systemPrompt = useTools ? SYSTEM_PROMPT_WITH_TOOLS : SYSTEM_PROMPT_NO_TOOLS;

    // Convert UIMessage[] → OpenAI format for WebLLM
    const openaiMessages: Array<{ role: string; content: string }> = [
      { role: 'system', content: systemPrompt },
      ...messages.map((m) => ({
        role: m.role,
        content: extractText(m),
      })),
    ];

    try {
      const response = await currentEngine.chat.completions.create({
        messages: openaiMessages,
        stream: true,
        ...(useTools ? { tools: MAP_TOOLS } : {}),
        temperature: 0.7,
        max_tokens: 1024,
      });

      let pendingToolCalls: Array<{
        id: string;
        function: { name: string; arguments: string };
      }> = [];

      // Accumulate full response text for non-tool models (to parse action blocks)
      let accumulatedText = '';

      for await (const chunk of response) {
        const choice = chunk.choices[0];
        if (!choice) continue;

        // Stream text content
        const delta = choice.delta?.content;
        if (delta) {
          accumulatedText += delta;
          yield { type: 'TEXT_MESSAGE_CONTENT' as const, messageId, delta, timestamp: Date.now() };
        }

        // Collect tool calls (WebLLM sends them on the last chunk only)
        if (choice.delta?.tool_calls) {
          for (const tc of choice.delta.tool_calls) {
            if (tc.id && tc.function) {
              pendingToolCalls.push({
                id: tc.id,
                function: {
                  name: tc.function.name ?? '',
                  arguments: tc.function.arguments ?? '',
                },
              });
            }
          }
        }
      }

      // --- Native tool calls (Hermes models) ---
      if (pendingToolCalls.length > 0) {
        // End the text message first, then handle tool calls as separate events
        yield { type: 'TEXT_MESSAGE_END' as const, messageId, timestamp: Date.now() };

        for (const toolCall of pendingToolCalls) {
          const toolCallId = toolCall.id;
          const toolName = toolCall.function.name;

          yield { type: 'TOOL_CALL_START' as const, toolCallId, toolName, timestamp: Date.now() };
          yield { type: 'TOOL_CALL_ARGS' as const, toolCallId, delta: toolCall.function.arguments, timestamp: Date.now() };
          yield { type: 'TOOL_CALL_END' as const, toolCallId, timestamp: Date.now() };

          let toolArgs: Record<string, unknown> = {};
          try {
            toolArgs = JSON.parse(toolCall.function.arguments);
          } catch {
            // Invalid JSON args
          }

          const result = executeMapTool(toolName, toolArgs, mapRef.value);
          const resultMsgId = crypto.randomUUID();
          yield { type: 'TEXT_MESSAGE_START' as const, messageId: resultMsgId, role: 'assistant' as const, timestamp: Date.now() };

          const resultText = typeof result === 'object' && result !== null && 'message' in result
            ? String((result as Record<string, unknown>).message)
            : JSON.stringify(result);

          yield { type: 'TEXT_MESSAGE_CONTENT' as const, messageId: resultMsgId, delta: `\u2705 ${resultText}`, timestamp: Date.now() };
          yield { type: 'TEXT_MESSAGE_END' as const, messageId: resultMsgId, timestamp: Date.now() };
        }
        pendingToolCalls = [];
      } else if (!useTools && accumulatedText) {
        // --- Text-based action parsing (Qwen / non-tool models) ---
        // Parse [MAP_ACTION] blocks and execute them, appending result to SAME message
        const { actions } = parseMapActions(accumulatedText);
        for (const action of actions) {
          const { action: toolName, ...toolArgs } = action;
          const result = executeMapTool(toolName, toolArgs, mapRef.value);

          const resultText = typeof result === 'object' && result !== null && 'message' in result
            ? String((result as Record<string, unknown>).message)
            : JSON.stringify(result);

          // Append action result to the SAME message (not a new one)
          yield { type: 'TEXT_MESSAGE_CONTENT' as const, messageId, delta: `\n\n\u2705 ${resultText}`, timestamp: Date.now() };
        }

        // End the message AFTER appending action results
        yield { type: 'TEXT_MESSAGE_END' as const, messageId, timestamp: Date.now() };
      } else {
        // No actions — just end the message
        yield { type: 'TEXT_MESSAGE_END' as const, messageId, timestamp: Date.now() };
      }
    } catch (err) {
      const errorDelta = err instanceof Error ? err.message : 'An error occurred';
      yield { type: 'TEXT_MESSAGE_CONTENT' as const, messageId, delta: `❌ Error: ${errorDelta}`, timestamp: Date.now() };
      yield { type: 'TEXT_MESSAGE_END' as const, messageId, timestamp: Date.now() };
    }

    yield { type: 'RUN_FINISHED' as const, runId, finishReason: 'stop' as const, timestamp: Date.now() };
  });

  const chat = useChat({
    connection,
  });

  // Also expose engine status for the UI
  return {
    ...chat,
    // @ts-expect-error - extending UseChatReturn with engine status
    engineStatus: status,
  };
}
