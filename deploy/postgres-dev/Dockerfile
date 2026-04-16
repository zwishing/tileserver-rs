# Lightweight PostgreSQL + PostGIS image for local development and OGC testing.
#
# Uses the official postgis/postgis image instead of building from source so the
# first `docker compose up` finishes quickly. The benchmarks/postgres Dockerfile
# stays as-is for benchmark runs that require a native-arch rebuild.

FROM postgis/postgis:16-3.4

ENV POSTGRES_USER=tileserver \
    POSTGRES_PASSWORD=tileserver \
    POSTGRES_DB=gis

HEALTHCHECK --interval=5s --timeout=3s --retries=10 \
  CMD pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB" || exit 1
