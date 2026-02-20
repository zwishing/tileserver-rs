export interface PingResponse {
  status: string;
  config_hash: string;
  loaded_at_unix: number;
  loaded_sources: number;
  loaded_styles: number;
  renderer_enabled: boolean;
  version: string;
}
