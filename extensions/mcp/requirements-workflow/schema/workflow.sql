CREATE TABLE IF NOT EXISTS requirements_workflows (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id),
    name TEXT NOT NULL,
    revision BIGINT NOT NULL DEFAULT 0,
    state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS requirements_workflow_events (
    workflow_id TEXT NOT NULL REFERENCES requirements_workflows(id),
    revision BIGINT NOT NULL,
    actor_id TEXT NOT NULL,
    event TEXT NOT NULL,
    state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workflow_id, revision)
);
CREATE TABLE IF NOT EXISTS requirements_workflow_keys (
    key_id TEXT PRIMARY KEY,
    public_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS requirements_workflow_review_nonces (
    nonce TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL REFERENCES requirements_workflows(id),
    revision BIGINT NOT NULL,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '30 minutes'
);

CREATE TABLE IF NOT EXISTS requirements_publication_attempts (
    workflow_id TEXT NOT NULL REFERENCES requirements_workflows(id),
    title TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('started','confirmed')),
    page_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(workflow_id,title)
);
