-- ============================================================================
-- OGC API Features benchmark data (cities / countries / roads)
-- NOTE: docker-entrypoint-initdb.d runs files in C-locale sort order, and
-- `init-ogc.sql` sorts BEFORE `init.sql` (because `-` = 0x2D < `.` = 0x2E).
-- So we defensively load PostGIS here too; `IF NOT EXISTS` makes this
-- idempotent when `init.sql` later runs the same statement.
-- ============================================================================

CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS postgis_topology;

CREATE TABLE IF NOT EXISTS cities (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  country TEXT NOT NULL,
  population BIGINT,
  geom GEOMETRY(Point, 4326)
);

CREATE TABLE IF NOT EXISTS countries (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  iso_code TEXT,
  continent TEXT,
  geom GEOMETRY(Polygon, 4326)
);

CREATE TABLE IF NOT EXISTS roads (
  id SERIAL PRIMARY KEY,
  name TEXT,
  road_type TEXT,
  geom GEOMETRY(LineString, 4326)
);

-- Populate cities: 500 rows with realistic world distribution
INSERT INTO cities (name, country, population, geom)
SELECT
  'City_' || i,
  (ARRAY['USA','DE','FR','JP','UK','BR','IN','CN','IT','ES'])[1 + (random()*9)::int],
  (500000 + random() * 15000000)::bigint,
  ST_SetSRID(ST_MakePoint(
    -180 + random() * 360,
    -60 + random() * 120
  ), 4326)
FROM generate_series(1, 500) AS i;

-- Populate countries: 50 rectangular regions (simplified)
INSERT INTO countries (name, iso_code, continent, geom)
SELECT
  'Country_' || i,
  'C' || lpad(i::text, 2, '0'),
  (ARRAY['EU','NA','SA','AS','AF','OC'])[1 + (random()*5)::int],
  ST_SetSRID(ST_MakeEnvelope(
    -170 + (i * 7)::int, -55 + (i * 3)::int % 100,
    -160 + (i * 7)::int, -50 + (i * 3)::int % 100
  ), 4326)
FROM generate_series(1, 50) AS i;

-- Populate roads: 1000 linestrings
INSERT INTO roads (name, road_type, geom)
SELECT
  'Road_' || i,
  (ARRAY['highway','primary','secondary','residential'])[1 + (random()*3)::int],
  ST_SetSRID(ST_MakeLine(
    ST_MakePoint(-180 + random() * 360, -60 + random() * 120),
    ST_MakePoint(-180 + random() * 360, -60 + random() * 120)
  ), 4326)
FROM generate_series(1, 1000) AS i;

-- Spatial indexes
CREATE INDEX IF NOT EXISTS cities_geom_idx ON cities USING GIST (geom);
CREATE INDEX IF NOT EXISTS countries_geom_idx ON countries USING GIST (geom);
CREATE INDEX IF NOT EXISTS roads_geom_idx ON roads USING GIST (geom);

-- Analyze tables
ANALYZE cities;
ANALYZE countries;
ANALYZE roads;

DO $$
BEGIN
  RAISE NOTICE 'OGC API Features benchmark tables initialized: cities (500), countries (50), roads (1000)';
END $$;
