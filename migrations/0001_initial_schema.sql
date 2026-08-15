-- Initial schema: sources, health, events, projects, crates, metrics, advisories.

CREATE TABLE source_health (
    source              TEXT PRIMARY KEY,
    status              TEXT NOT NULL DEFAULT 'disabled',
    last_fetched_at     TIMESTAMPTZ,
    last_attempted_at   TIMESTAMPTZ,
    last_error          TEXT,
    last_duration_ms    BIGINT,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE events (
    id              UUID PRIMARY KEY,
    source          TEXT NOT NULL,
    source_id       TEXT NOT NULL,
    event_type      TEXT NOT NULL,
    title           TEXT NOT NULL,
    url             TEXT NOT NULL,
    summary         TEXT,
    published_at    TIMESTAMPTZ,
    fetched_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Idempotency: the same source item must never be inserted twice.
    UNIQUE (source, source_id)
);

CREATE INDEX events_published_idx ON events (published_at DESC NULLS LAST);
CREATE INDEX events_type_idx ON events (event_type);
CREATE INDEX events_fetched_idx ON events (fetched_at);

CREATE TABLE event_provenance (
    event_id        UUID NOT NULL REFERENCES events (id) ON DELETE CASCADE,
    source          TEXT NOT NULL,
    source_id       TEXT NOT NULL,
    source_url      TEXT NOT NULL,
    published_at    TIMESTAMPTZ,
    fetched_at      TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (event_id, source, source_id)
);

CREATE TABLE projects (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    html_url        TEXT NOT NULL,
    stars           BIGINT NOT NULL DEFAULT 0,
    forks           BIGINT NOT NULL DEFAULT 0,
    open_issues     BIGINT NOT NULL DEFAULT 0,
    language        TEXT,
    archived        BOOLEAN NOT NULL DEFAULT false,
    pushed_at       TIMESTAMPTZ,
    fetched_at      TIMESTAMPTZ NOT NULL
);

CREATE INDEX projects_stars_idx ON projects (stars DESC) WHERE archived = false;

CREATE TABLE crates (
    id                  TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    description         TEXT,
    homepage            TEXT,
    repository          TEXT,
    downloads           BIGINT NOT NULL DEFAULT 0,
    recent_downloads    BIGINT NOT NULL DEFAULT 0,
    max_version         TEXT NOT NULL,
    updated_at          TIMESTAMPTZ,
    fetched_at          TIMESTAMPTZ NOT NULL
);

CREATE INDEX crates_recent_downloads_idx ON crates (recent_downloads DESC);

-- Time-series observations for velocity / momentum (AGENTS.md §19).
CREATE TABLE metrics (
    id              BIGSERIAL PRIMARY KEY,
    entity_type     TEXT NOT NULL,
    entity_id       TEXT NOT NULL,
    metric          TEXT NOT NULL,
    value           DOUBLE PRECISION NOT NULL,
    observed_at     TIMESTAMPTZ NOT NULL,
    UNIQUE (entity_type, entity_id, metric, observed_at)
);

CREATE INDEX metrics_series_idx ON metrics (entity_type, entity_id, metric, observed_at);

CREATE TABLE advisories (
    id                  TEXT PRIMARY KEY,
    title               TEXT NOT NULL,
    url                 TEXT NOT NULL,
    summary             TEXT,
    affected_crates     JSONB NOT NULL DEFAULT '[]',
    published_at        TIMESTAMPTZ,
    fetched_at          TIMESTAMPTZ NOT NULL
);

CREATE INDEX advisories_published_idx ON advisories (published_at DESC NULLS LAST);
