CREATE TABLE IF NOT EXISTS location_page_views (
  day TEXT NOT NULL,
  route TEXT NOT NULL,
  country_code TEXT NOT NULL,
  region_code TEXT NOT NULL,
  views INTEGER NOT NULL CHECK (views >= 0),
  updated_at TEXT NOT NULL,
  PRIMARY KEY (day, route, country_code, region_code)
);

CREATE INDEX IF NOT EXISTS location_page_views_day
  ON location_page_views (day);
