-- Marketplace content hash as version identity. Twin of
-- schema/36_marketplace_versions.sql, plus: seed legacy versions from the
-- invocation facts already retained (their source hash stands in for the
-- marketplace hash), and stamp every new invocation fact with the current
-- marketplace hash.

CREATE TABLE IF NOT EXISTS marketplace_versions (
    marketplace_id TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    source TEXT NOT NULL,
    source_hash TEXT,
    manifest JSONB,
    plugin_count INTEGER NOT NULL DEFAULT 0,
    skill_count INTEGER NOT NULL DEFAULT 0,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    effective_until TIMESTAMPTZ,
    origin TEXT NOT NULL DEFAULT 'manifest' CHECK (origin IN ('manifest', 'legacy_source_hash')),
    PRIMARY KEY (marketplace_id, content_hash)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_marketplace_versions_current
    ON marketplace_versions(marketplace_id) WHERE effective_until IS NULL;
CREATE INDEX IF NOT EXISTS idx_marketplace_versions_interval
    ON marketplace_versions(marketplace_id, first_seen_at, effective_until);

CREATE OR REPLACE FUNCTION marketplace_version_at(mid TEXT, at TIMESTAMPTZ)
RETURNS TEXT LANGUAGE sql STABLE AS $$
SELECT content_hash FROM marketplace_versions
WHERE marketplace_id = mid AND first_seen_at <= at
ORDER BY (effective_until IS NULL OR at < effective_until) DESC, first_seen_at DESC
LIMIT 1
$$;

-- Facts made before this table carry the source hash their skill ran from.
-- Each distinct (marketplace, source hash) becomes a closed legacy version
-- spanning the facts that carried it, so history reaches back as far as the
-- retained facts do.
INSERT INTO marketplace_versions
    (marketplace_id, content_hash, source, source_hash, origin, first_seen_at, last_seen_at, effective_until)
SELECT fact->'value'->'skill'->>'marketplace_id',
       fact->'value'->'skill'->>'source_hash',
       coalesce(fact->'value'->'skill'->>'source', 'base'),
       fact->'value'->'skill'->>'source_hash',
       'legacy_source_hash', min(occurred_at), max(occurred_at), max(occurred_at)
FROM analytics_normalized_facts
WHERE NOT deleted AND fact_kind = 'invocation'
  AND fact->'value'->'skill'->>'marketplace_id' IS NOT NULL
  AND fact->'value'->'skill'->>'source_hash' IS NOT NULL
GROUP BY 1, 2, 3, 4
ON CONFLICT DO NOTHING;

-- Same body as schema/31_feedback_projection.sql: adds marketplace_hash.
CREATE OR REPLACE FUNCTION feedback_event_skill(event_key TEXT)
RETURNS JSONB LANGUAGE sql STABLE AS $$
SELECT CASE WHEN e.plugin_id IS NULL OR e.skill IS NULL OR e.skill='' THEN NULL
 ELSE jsonb_strip_nulls(jsonb_build_object('plugin_id',e.plugin_id,'skill',e.skill,
  'marketplace_id',o.marketplace_id,'source',o.source,'source_hash',s.content_hash,
  'marketplace_hash',v.content_hash)) END
FROM analysis_skill_events e
LEFT JOIN service_owned_ids o ON o.kind='plugin' AND o.id=e.plugin_id
LEFT JOIN service_sources s ON s.name=o.source
LEFT JOIN marketplace_versions v ON v.marketplace_id=o.marketplace_id AND v.effective_until IS NULL
WHERE e.id=event_key
$$;
