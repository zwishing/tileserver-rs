# Deployment manifests

Each subdirectory targets one deployment scenario. They share the same base `compose.yml` shape but ship different overrides + entrypoints. Pick the one that matches what you're doing.

## `local/` — laptop development

PostgreSQL + PostGIS sidecar with seeded sample data, hot-reload-friendly volume mounts, port bound to `127.0.0.1` only.

```bash
docker compose \
  -f deploy/local/compose.yml \
  -f deploy/local/compose.override.yml \
  up -d
```

Backed by `data/postgres-dev/init.sql` (cities, countries, roads, buildings) and `data/configs/dev-postgres.toml`.

## `prod/` — production / single-node

Pulls the published `ghcr.io/vinayakkulkarni/tileserver-rs:latest` image, exposes on `${HOST:-0.0.0.0}:${PORT:-8080}`, sets resource limits, expects a `config.toml` next to the compose file.

```bash
docker compose \
  -f deploy/prod/compose.yml \
  -f deploy/prod/compose.override.yml \
  up -d
```

For Kubernetes/Fly/Railway/Render, see `apps/docs/content/4.guides/9.deploy.md` — those platforms don't use Docker Compose.

## `benchmarks/` — perf testing

Pointer to the canonical benchmarking stack at the repo root.

See [`benchmarks/`](../benchmarks/) for the full multi-server harness (tileserver-rs, tileserver-gl, martin, titiler, postgres) and `benchmarks/docker-compose.yml` profiles.
