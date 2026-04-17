-- PostgreSQL/PostGIS initialization for tileserver-rs local development.
--
-- Seeds small but realistic OGC API Features test data:
--   - public.cities      (POINT, 4326): a handful of world cities
--   - public.countries   (MULTIPOLYGON, 4326): simplified bboxes as polygons
--   - public.roads       (LINESTRING, 4326): a few sample roads for LineString support
--   - public.buildings   (POLYGON, 4326): toy buildings for OGC items pagination
--
-- This is designed to exercise:
--   * OGC API Features Part 1 Core (collections, items, single feature)
--   * Part 2 CRS (4326 + 3857 reprojection)
--   * Part 3 CQL2 filtering (=, <, >, BETWEEN, LIKE, IN, IS NULL, AND/OR, INTERSECTS)
--   * Part 4 Transactions (writable table with a primary key)
--   * Part 5 Schemas (varied column types: text, int, bool, float, timestamp)

CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS postgis_topology;

-- ============================================================================
-- cities (POINT, 4326)
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.cities (
    id           SERIAL PRIMARY KEY,
    name         TEXT NOT NULL,
    country      TEXT NOT NULL,
    population   BIGINT,
    is_capital   BOOLEAN NOT NULL DEFAULT FALSE,
    founded_year INTEGER,
    elevation_m  DOUBLE PRECISION,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    geom         GEOMETRY(Point, 4326) NOT NULL
);
CREATE INDEX IF NOT EXISTS cities_geom_idx ON public.cities USING GIST (geom);

INSERT INTO public.cities (name, country, population, is_capital, founded_year, elevation_m, geom) VALUES
    ('San Francisco', 'United States',   873965, FALSE, 1776,   16, ST_SetSRID(ST_MakePoint(-122.4194,  37.7749), 4326)),
    ('New York',      'United States',  8336817, FALSE, 1624,   10, ST_SetSRID(ST_MakePoint( -74.0060,  40.7128), 4326)),
    ('Washington',    'United States',   712816, TRUE,  1790,    0, ST_SetSRID(ST_MakePoint( -77.0369,  38.9072), 4326)),
    ('London',        'United Kingdom', 8982000, TRUE,   43,    11, ST_SetSRID(ST_MakePoint(  -0.1276,  51.5074), 4326)),
    ('Paris',         'France',         2148000, TRUE,  259,    35, ST_SetSRID(ST_MakePoint(   2.3522,  48.8566), 4326)),
    ('Berlin',        'Germany',        3769000, TRUE,  1237,   34, ST_SetSRID(ST_MakePoint(  13.4050,  52.5200), 4326)),
    ('Tokyo',         'Japan',         13960000, TRUE,  1457,   40, ST_SetSRID(ST_MakePoint( 139.6917,  35.6895), 4326)),
    ('Mumbai',        'India',         20410000, FALSE, 1507,   14, ST_SetSRID(ST_MakePoint(  72.8777,  19.0760), 4326)),
    ('Delhi',         'India',         32900000, TRUE,  NULL,  216, ST_SetSRID(ST_MakePoint(  77.2090,  28.6139), 4326)),
    ('Sydney',        'Australia',      5312000, FALSE, 1788,    3, ST_SetSRID(ST_MakePoint( 151.2093, -33.8688), 4326)),
    ('Cape Town',     'South Africa',   4618000, FALSE, 1652,    0, ST_SetSRID(ST_MakePoint(  18.4241, -33.9249), 4326)),
    ('São Paulo',     'Brazil',        12330000, FALSE, 1554,  760, ST_SetSRID(ST_MakePoint( -46.6333, -23.5505), 4326));

-- ============================================================================
-- countries (MULTIPOLYGON, 4326) — real Natural Earth outlines, simplified to
-- ~0.05° tolerance to keep the seed file under 300 KB. The full-resolution
-- shapes are in natural_earth.sqlite (Natural Earth 10m admin_0_countries,
-- public domain). See deploy/postgres-dev/countries_seed.sql for the data.
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.countries (
    id          SERIAL PRIMARY KEY,
    iso_a2      CHAR(2) NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    continent   TEXT NOT NULL,
    population  BIGINT,
    geom        GEOMETRY(MultiPolygon, 4326) NOT NULL
);
CREATE INDEX IF NOT EXISTS countries_geom_idx ON public.countries USING GIST (geom);

\i /docker-entrypoint-initdb.d/countries_seed.sql

-- ============================================================================
-- roads (LINESTRING, 4326) — a few demo lines
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.roads (
    id        SERIAL PRIMARY KEY,
    name      TEXT NOT NULL,
    road_type TEXT NOT NULL,
    maxspeed  INTEGER,
    geom      GEOMETRY(LineString, 4326) NOT NULL
);
CREATE INDEX IF NOT EXISTS roads_geom_idx ON public.roads USING GIST (geom);

INSERT INTO public.roads (name, road_type, maxspeed, geom) VALUES
    ('Market Street',     'primary',      50, ST_SetSRID(ST_MakeLine(ARRAY[
        ST_MakePoint(-122.4194, 37.7749),
        ST_MakePoint(-122.4080, 37.7840),
        ST_MakePoint(-122.3965, 37.7930)
    ]), 4326)),
    ('Bay Bridge',        'motorway',    100, ST_SetSRID(ST_MakeLine(ARRAY[
        ST_MakePoint(-122.3873, 37.7937),
        ST_MakePoint(-122.3200, 37.8200),
        ST_MakePoint(-122.2700, 37.8300)
    ]), 4326)),
    ('Broadway',          'secondary',    40, ST_SetSRID(ST_MakeLine(ARRAY[
        ST_MakePoint(-74.0060, 40.7128),
        ST_MakePoint(-73.9857, 40.7580),
        ST_MakePoint(-73.9800, 40.7620)
    ]), 4326));

-- ============================================================================
-- buildings (POLYGON, 4326) — many small rows to exercise pagination
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.buildings (
    id        SERIAL PRIMARY KEY,
    name      TEXT,
    height_m  DOUBLE PRECISION,
    floors    INTEGER,
    building  TEXT NOT NULL DEFAULT 'yes',
    geom      GEOMETRY(Polygon, 4326) NOT NULL
);
CREATE INDEX IF NOT EXISTS buildings_geom_idx ON public.buildings USING GIST (geom);

INSERT INTO public.buildings (name, height_m, floors, building, geom)
SELECT
    'Building ' || i,
    10 + (i % 20) * 5,
    1 + (i % 20),
    CASE (i % 4)
        WHEN 0 THEN 'residential'
        WHEN 1 THEN 'commercial'
        WHEN 2 THEN 'industrial'
        ELSE 'yes'
    END,
    ST_SetSRID(
        ST_MakeEnvelope(
            -122.42 + (i * 0.001) - 0.0005,
              37.77 + (i * 0.0007) - 0.0003,
            -122.42 + (i * 0.001) + 0.0005,
              37.77 + (i * 0.0007) + 0.0003,
            4326
        ),
        4326
    )
FROM generate_series(1, 250) AS i;

ANALYZE public.cities;
ANALYZE public.countries;
ANALYZE public.roads;
ANALYZE public.buildings;

DO $$
DECLARE
    cities_count     integer;
    countries_count  integer;
    roads_count      integer;
    buildings_count  integer;
BEGIN
    SELECT count(*) INTO cities_count    FROM public.cities;
    SELECT count(*) INTO countries_count FROM public.countries;
    SELECT count(*) INTO roads_count     FROM public.roads;
    SELECT count(*) INTO buildings_count FROM public.buildings;

    RAISE NOTICE 'Dev PostGIS seeded:';
    RAISE NOTICE '  cities:    % rows', cities_count;
    RAISE NOTICE '  countries: % rows', countries_count;
    RAISE NOTICE '  roads:     % rows', roads_count;
    RAISE NOTICE '  buildings: % rows', buildings_count;
END $$;
