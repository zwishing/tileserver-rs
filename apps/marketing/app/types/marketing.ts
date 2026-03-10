import type { Component } from 'vue';

export interface Feature {
  icon: Component;
  title: string;
  description: string;
}

export interface ApiEndpoint {
  method: string;
  path: string;
}

export interface ApiEndpointGroup {
  title: string;
  endpoints: ApiEndpoint[];
}

export interface PerformanceStat {
  icon: Component;
  value: number;
  label: string;
  detail: string;
  prefix?: string;
}

export interface AiBenefit {
  icon: Component;
  title: string;
  description: string;
}

export interface AiChatMessage {
  role: 'user' | 'assistant';
  text: string;
}
