-- One Salesforce Username per (user, org). `provider` is the MCP server id
-- fronting the org; existing rows belong to the default `salesforce` server.
ALTER TABLE salesforce_user_identities
    ADD COLUMN IF NOT EXISTS provider TEXT NOT NULL DEFAULT 'salesforce';
ALTER TABLE salesforce_user_identities
    DROP CONSTRAINT IF EXISTS salesforce_user_identities_provider_check;
ALTER TABLE salesforce_user_identities
    ADD CONSTRAINT salesforce_user_identities_provider_check
    CHECK (provider ~ '^salesforce(-[A-Za-z0-9_-]{1,117})?$');
ALTER TABLE salesforce_user_identities
    DROP CONSTRAINT IF EXISTS salesforce_user_identities_pkey;
ALTER TABLE salesforce_user_identities
    ADD PRIMARY KEY (user_id, provider);
