/**
 * Map Tool Definitions
 *
 * Defines all client-side map tools using TanStack AI's `toolDefinition()` with zod schemas.
 * Each tool definition is used for:
 *   1. Type-safe client tool execution via `.client(handler)`
 *   2. Generating OpenAI-format tool schemas for WebLLM
 *
 * @see https://tanstack.com/ai/latest/docs/guides/client-tools
 * @see https://tanstack.com/ai/latest/docs/reference/interfaces/ClientTool
 */

import { toolDefinition } from '@tanstack/ai';
import { z } from 'zod';
import type { Map as MaplibreMap, FilterSpecification } from 'maplibre-gl';

// =============================================================================
// TOOL DEFINITIONS (zod schemas + metadata)
// =============================================================================

export const flyToDef = toolDefinition({
  name: 'fly_to',
  description: 'Animate the map camera to a specific location. Use when the user asks to go to, show, or navigate to a place.',
  inputSchema: z.object({
    lng: z.number().describe('Longitude (-180 to 180)'),
    lat: z.number().describe('Latitude (-90 to 90)'),
    zoom: z.number().optional().describe('Zoom level (0-22, default 12)'),
    bearing: z.number().optional().describe('Bearing in degrees (default 0)'),
    pitch: z.number().optional().describe('Pitch in degrees 0-85 (default 0)'),
  }),
  outputSchema: z.object({
    success: z.boolean(),
    message: z.string(),
  }),
});

export const fitBoundsDef = toolDefinition({
  name: 'fit_bounds',
  description: 'Fit the map camera to a bounding box. Use when showing a region, country, or area.',
  inputSchema: z.object({
    west: z.number().describe('West longitude'),
    south: z.number().describe('South latitude'),
    east: z.number().describe('East longitude'),
    north: z.number().describe('North latitude'),
    padding: z.number().optional().describe('Padding in pixels (default 50)'),
  }),
  outputSchema: z.object({
    success: z.boolean(),
    message: z.string(),
  }),
});

export const getMapStateDef = toolDefinition({
  name: 'get_map_state',
  description: 'Get the current map center, zoom, bearing, pitch, and visible layers. Use to understand what the user is looking at.',
  inputSchema: z.object({}),
  outputSchema: z.object({
    center: z.object({ lng: z.number(), lat: z.number() }),
    zoom: z.number(),
    bearing: z.number(),
    pitch: z.number(),
    visibleLayers: z.array(z.string()),
  }),
});

export const setLayerVisibilityDef = toolDefinition({
  name: 'set_layer_visibility',
  description: 'Show or hide a map layer by its ID.',
  inputSchema: z.object({
    layerId: z.string().describe('The layer ID to toggle'),
    visible: z.boolean().describe('true to show, false to hide'),
  }),
  outputSchema: z.object({
    success: z.boolean(),
    message: z.string(),
  }),
});

export const setLayerPaintDef = toolDefinition({
  name: 'set_layer_paint',
  description: 'Change a paint property of a map layer (e.g., color, opacity, width).',
  inputSchema: z.object({
    layerId: z.string().describe('The layer ID to modify'),
    property: z.string().describe('Paint property name (e.g., fill-color, line-width, fill-opacity)'),
    value: z.union([z.string(), z.number(), z.array(z.unknown())]).describe('New value for the property'),
  }),
  outputSchema: z.object({
    success: z.boolean(),
    message: z.string(),
  }),
});

export const setLayerFilterDef = toolDefinition({
  name: 'set_layer_filter',
  description: 'Apply a MapLibre filter expression to a layer to show only matching features.',
  inputSchema: z.object({
    layerId: z.string().describe('The layer ID to filter'),
    filter: z.array(z.unknown()).describe('MapLibre filter expression (e.g., ["==", "type", "park"])'),
  }),
  outputSchema: z.object({
    success: z.boolean(),
    message: z.string(),
  }),
});

export const queryRenderedFeaturesDef = toolDefinition({
  name: 'query_rendered_features',
  description: 'Query features visible in the current map viewport. Returns feature properties and geometry type.',
  inputSchema: z.object({
    layers: z.array(z.string()).optional().describe('Layer IDs to query (omit for all layers)'),
    limit: z.number().optional().describe('Max features to return (default 10)'),
  }),
  outputSchema: z.object({
    features: z.array(z.object({
      layer: z.string(),
      geometryType: z.string(),
      properties: z.record(z.unknown()),
    })),
    total: z.number(),
  }),
});

export const addHighlightDef = toolDefinition({
  name: 'add_highlight',
  description: 'Temporarily highlight features on the map matching a filter. Highlight auto-removes after 8 seconds.',
  inputSchema: z.object({
    layerId: z.string().describe('Source layer ID to highlight features from'),
    filter: z.array(z.unknown()).describe('MapLibre filter expression for features to highlight'),
    color: z.string().optional().describe('Highlight color (default "#ff0000")'),
  }),
  outputSchema: z.object({
    success: z.boolean(),
    message: z.string(),
  }),
});

export const generateStyleDef = toolDefinition({
  name: 'generate_style',
  description: 'Modify the current map style based on a description (e.g., "make the water blue", "dark mode"). Adjusts paint properties of matching layers.',
  inputSchema: z.object({
    description: z.string().describe('Natural language description of style changes'),
    changes: z.array(z.object({
      layerId: z.string().describe('Layer ID to modify'),
      property: z.string().describe('Paint property to change'),
      value: z.union([z.string(), z.number(), z.array(z.unknown())]).describe('New value'),
    })).describe('Array of specific paint property changes to apply'),
  }),
  outputSchema: z.object({
    success: z.boolean(),
    message: z.string(),
    changesApplied: z.number(),
  }),
});

// =============================================================================
// ALL TOOL DEFINITIONS (for iteration)
// =============================================================================

export const ALL_TOOL_DEFINITIONS = [
  flyToDef,
  fitBoundsDef,
  getMapStateDef,
  setLayerVisibilityDef,
  setLayerPaintDef,
  setLayerFilterDef,
  queryRenderedFeaturesDef,
  addHighlightDef,
  generateStyleDef,
] as const;

// =============================================================================
// CLIENT TOOL FACTORY
// =============================================================================

/**
 * Create client tool implementations bound to a MapLibre map instance.
 * Returns an array of ClientTool objects to pass to `useChat({ tools })`.
 *
 * @param getMap - Function that returns the current map instance (or null)
 */
export function createMapClientTools(getMap: () => MaplibreMap | null) {
  const flyTo = flyToDef.client(({ lng, lat, zoom, bearing, pitch }) => {
    const map = getMap();
    if (!map) return { success: false, message: 'Map not available' };
    map.flyTo({
      center: [lng, lat],
      zoom: zoom ?? 12,
      bearing: bearing ?? 0,
      pitch: pitch ?? 0,
      duration: 2000,
    });
    return { success: true, message: `Flying to [${lng}, ${lat}] at zoom ${zoom ?? 12}` };
  });

  const fitBounds = fitBoundsDef.client(({ west, south, east, north, padding }) => {
    const map = getMap();
    if (!map) return { success: false, message: 'Map not available' };
    map.fitBounds([[west, south], [east, north]], { padding: padding ?? 50, duration: 2000 });
    return { success: true, message: `Fitting to bounds [${west},${south},${east},${north}]` };
  });

  const getMapState = getMapStateDef.client(() => {
    const map = getMap();
    if (!map) {
      return {
        center: { lng: 0, lat: 0 },
        zoom: 0,
        bearing: 0,
        pitch: 0,
        visibleLayers: [],
      };
    }
    const center = map.getCenter();
    const layers = map.getStyle()?.layers?.map((l) => l.id).slice(0, 30) ?? [];
    return {
      center: { lng: Math.round(center.lng * 1000) / 1000, lat: Math.round(center.lat * 1000) / 1000 },
      zoom: Math.round(map.getZoom() * 100) / 100,
      bearing: Math.round(map.getBearing()),
      pitch: Math.round(map.getPitch()),
      visibleLayers: layers,
    };
  });

  const setLayerVisibility = setLayerVisibilityDef.client(({ layerId, visible }) => {
    const map = getMap();
    if (!map) return { success: false, message: 'Map not available' };
    try {
      map.setLayoutProperty(layerId, 'visibility', visible ? 'visible' : 'none');
      return { success: true, message: `Layer "${layerId}" ${visible ? 'shown' : 'hidden'}` };
    } catch (err) {
      return { success: false, message: `Failed to toggle layer: ${err instanceof Error ? err.message : String(err)}` };
    }
  });

  const setLayerPaint = setLayerPaintDef.client(({ layerId, property, value }) => {
    const map = getMap();
    if (!map) return { success: false, message: 'Map not available' };
    try {
      map.setPaintProperty(layerId, property, value);
      return { success: true, message: `Set ${property} = ${JSON.stringify(value)} on "${layerId}"` };
    } catch (err) {
      return { success: false, message: `Failed to set paint: ${err instanceof Error ? err.message : String(err)}` };
    }
  });

  const setLayerFilter = setLayerFilterDef.client(({ layerId, filter }) => {
    const map = getMap();
    if (!map) return { success: false, message: 'Map not available' };
    try {
      map.setFilter(layerId, filter as FilterSpecification);
      return { success: true, message: `Filter applied to "${layerId}"` };
    } catch (err) {
      return { success: false, message: `Failed to set filter: ${err instanceof Error ? err.message : String(err)}` };
    }
  });

  const queryRenderedFeatures = queryRenderedFeaturesDef.client(({ layers, limit }) => {
    const map = getMap();
    if (!map) return { features: [], total: 0 };
    const maxFeatures = limit ?? 10;
    const opts: { layers?: string[] } = {};
    if (layers?.length) opts.layers = layers;
    const features = map.queryRenderedFeatures(undefined, opts);
    return {
      features: features.slice(0, maxFeatures).map((f) => ({
        layer: f.layer?.id ?? 'unknown',
        geometryType: f.geometry.type,
        properties: f.properties ?? {},
      })),
      total: features.length,
    };
  });

  const addHighlight = addHighlightDef.client(({ layerId, filter, color }) => {
    const map = getMap();
    if (!map) return { success: false, message: 'Map not available' };

    const highlightId = `__highlight_${Date.now()}`;
    const highlightColor = color ?? '#ff0000';

    try {
      // Find the source of the target layer
      const targetLayer = map.getStyle()?.layers?.find((l) => l.id === layerId);
      if (!targetLayer || !('source' in targetLayer)) {
        return { success: false, message: `Layer "${layerId}" not found or has no source` };
      }

      const sourceLayer = 'source-layer' in targetLayer ? targetLayer['source-layer'] : undefined;

      map.addLayer({
        id: highlightId,
        type: 'circle',
        source: targetLayer.source as string,
        ...(sourceLayer ? { 'source-layer': sourceLayer } : {}),
        filter: filter as FilterSpecification,
        paint: {
          'circle-radius': 8,
          'circle-color': highlightColor,
          'circle-opacity': 0.7,
          'circle-stroke-width': 2,
          'circle-stroke-color': '#ffffff',
        },
      });

      // Auto-remove after 8 seconds
      setTimeout(() => {
        try {
          if (map.getLayer(highlightId)) map.removeLayer(highlightId);
        } catch {
          // Layer may already be removed
        }
      }, 8000);

      return { success: true, message: `Highlighted features on "${layerId}" in ${highlightColor} (auto-removes in 8s)` };
    } catch (err) {
      return { success: false, message: `Failed to highlight: ${err instanceof Error ? err.message : String(err)}` };
    }
  });

  const generateStyle = generateStyleDef.client(({ description, changes }) => {
    const map = getMap();
    if (!map) return { success: false, message: 'Map not available', changesApplied: 0 };

    let applied = 0;
    for (const change of changes) {
      try {
        map.setPaintProperty(change.layerId, change.property, change.value);
        applied++;
      } catch {
        // Skip layers that don't exist or invalid properties
      }
    }

    return {
      success: applied > 0,
      message: `Applied ${applied}/${changes.length} style changes: "${description}"`,
      changesApplied: applied,
    };
  });

  return [
    flyTo,
    fitBounds,
    getMapState,
    setLayerVisibility,
    setLayerPaint,
    setLayerFilter,
    queryRenderedFeatures,
    addHighlight,
    generateStyle,
  ];
}

// =============================================================================
// OPENAI-FORMAT TOOLS (for WebLLM)
// =============================================================================

/**
 * OpenAI-compatible tool definitions for WebLLM's chat.completions.create().
 * These mirror the zod definitions above but in JSON Schema format.
 */
export const WEBLLM_TOOLS: Array<{
  type: 'function';
  function: {
    name: string;
    description: string;
    parameters: Record<string, unknown>;
  };
}> = [
  {
    type: 'function',
    function: {
      name: 'fly_to',
      description: 'Animate the map camera to a specific location. Use when the user asks to go to, show, or navigate to a place.',
      parameters: {
        type: 'object',
        properties: {
          lng: { type: 'number', description: 'Longitude (-180 to 180)' },
          lat: { type: 'number', description: 'Latitude (-90 to 90)' },
          zoom: { type: 'number', description: 'Zoom level (0-22, default 12)' },
          bearing: { type: 'number', description: 'Bearing in degrees (default 0)' },
          pitch: { type: 'number', description: 'Pitch in degrees 0-85 (default 0)' },
        },
        required: ['lng', 'lat'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'fit_bounds',
      description: 'Fit the map camera to a bounding box. Use when showing a region, country, or area.',
      parameters: {
        type: 'object',
        properties: {
          west: { type: 'number', description: 'West longitude' },
          south: { type: 'number', description: 'South latitude' },
          east: { type: 'number', description: 'East longitude' },
          north: { type: 'number', description: 'North latitude' },
          padding: { type: 'number', description: 'Padding in pixels (default 50)' },
        },
        required: ['west', 'south', 'east', 'north'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'get_map_state',
      description: 'Get the current map center, zoom, bearing, pitch, and visible layers.',
      parameters: { type: 'object', properties: {} },
    },
  },
  {
    type: 'function',
    function: {
      name: 'set_layer_visibility',
      description: 'Show or hide a map layer by its ID.',
      parameters: {
        type: 'object',
        properties: {
          layerId: { type: 'string', description: 'The layer ID to toggle' },
          visible: { type: 'boolean', description: 'true to show, false to hide' },
        },
        required: ['layerId', 'visible'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'set_layer_paint',
      description: 'Change a paint property of a map layer (e.g., color, opacity, width).',
      parameters: {
        type: 'object',
        properties: {
          layerId: { type: 'string', description: 'The layer ID to modify' },
          property: { type: 'string', description: 'Paint property name (e.g., fill-color, line-width)' },
          value: { description: 'New value for the property (string, number, or array)' },
        },
        required: ['layerId', 'property', 'value'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'set_layer_filter',
      description: 'Apply a MapLibre filter expression to a layer.',
      parameters: {
        type: 'object',
        properties: {
          layerId: { type: 'string', description: 'The layer ID to filter' },
          filter: { type: 'array', description: 'MapLibre filter expression (e.g., ["==", "type", "park"])' },
        },
        required: ['layerId', 'filter'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'query_rendered_features',
      description: 'Query features visible in the current map viewport.',
      parameters: {
        type: 'object',
        properties: {
          layers: { type: 'array', items: { type: 'string' }, description: 'Layer IDs to query (omit for all)' },
          limit: { type: 'number', description: 'Max features to return (default 10)' },
        },
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'add_highlight',
      description: 'Temporarily highlight features on the map matching a filter.',
      parameters: {
        type: 'object',
        properties: {
          layerId: { type: 'string', description: 'Source layer ID to highlight features from' },
          filter: { type: 'array', description: 'MapLibre filter expression for features to highlight' },
          color: { type: 'string', description: 'Highlight color (default "#ff0000")' },
        },
        required: ['layerId', 'filter'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'generate_style',
      description: 'Modify the current map style based on a description. Adjusts paint properties of matching layers.',
      parameters: {
        type: 'object',
        properties: {
          description: { type: 'string', description: 'Natural language description of style changes' },
          changes: {
            type: 'array',
            items: {
              type: 'object',
              properties: {
                layerId: { type: 'string' },
                property: { type: 'string' },
                value: { description: 'New value' },
              },
              required: ['layerId', 'property', 'value'],
            },
            description: 'Array of paint property changes to apply',
          },
        },
        required: ['description', 'changes'],
      },
    },
  },
];
