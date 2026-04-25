# `deploy/benchmarks/` — pointer

The benchmarking stack lives in [`benchmarks/`](../../benchmarks/) at the repo root, not here.

It's kept separate because:

- It pulls in three competitor images (tileserver-gl, martin, titiler) plus a full PostgreSQL/PostGIS instance. Lumping it into `deploy/` would make `docker compose -f deploy/.../compose.yml up` ambiguous about whether it includes those.
- The harness has its own `package.json`, autocannon driver, results writer, and grid-mode generator. None of that belongs in a generic deployment manifest.

To run benchmarks:

```bash
docker compose -f benchmarks/docker-compose.yml --profile all up -d
node benchmarks/run-benchmarks.js --type pmtiles --markdown
node benchmarks/run-benchmarks.js --mode grid --type pmtiles --grid-size 4x4
```

For full options, see `benchmarks/README.md` and `apps/docs/content/2.benchmarks/`.
