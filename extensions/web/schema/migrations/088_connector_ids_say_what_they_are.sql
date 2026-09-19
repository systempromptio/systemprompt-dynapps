-- The connector server ids are the only thing a Claude client shows the model,
-- so they must say what the connector is: `google_drive` read as "Drive only"
-- and the model refused to search Gmail through it, and `salesforce-uk` read
-- as a website rather than the CRM. The ids become `google_workspace` and
-- `salesforce-crm-<org>`. Identity rows and the seeded access rules follow the
-- rename so they keep matching the yaml. OAuth grants do not: the sealed grant
-- binds its provider inside the ciphertext, so a renamed grant would fail the
-- owner check on every call. Those connections are removed and each user
-- connects the renamed connector once more.

CREATE TEMP TABLE connector_id_renames (old_id TEXT PRIMARY KEY, new_id TEXT NOT NULL);
INSERT INTO connector_id_renames VALUES
    ('google_drive', 'google_workspace'),
    ('salesforce-uk', 'salesforce-crm-uk'),
    ('salesforce-us', 'salesforce-crm-us'),
    ('salesforce-dev', 'salesforce-crm-dev');

DELETE FROM mcp_connector_credentials c USING connector_id_renames r WHERE c.provider = r.old_id;
DELETE FROM mcp_connector_accounts a USING connector_id_renames r WHERE a.provider = r.old_id;
DELETE FROM mcp_connector_oauth_states s USING connector_id_renames r WHERE s.provider = r.old_id;
UPDATE salesforce_user_identities i SET provider = r.new_id
    FROM connector_id_renames r WHERE i.provider = r.old_id;

-- The rules' entity FK is neither deferrable nor ON UPDATE CASCADE, so the
-- entity row is copied under its new id, the rules move across, and the old
-- row goes last (its cascade then has nothing left to delete).
INSERT INTO access_control_entities (entity_type, entity_id, default_included, source, created_at, updated_at)
    SELECT e.entity_type, r.new_id, e.default_included, e.source, e.created_at, now()
    FROM access_control_entities e JOIN connector_id_renames r
        ON e.entity_type = 'mcp_server' AND e.entity_id = r.old_id
    ON CONFLICT (entity_type, entity_id) DO NOTHING;
UPDATE access_control_rules x SET entity_id = r.new_id
    FROM connector_id_renames r WHERE x.entity_type = 'mcp_server' AND x.entity_id = r.old_id;
DELETE FROM access_control_entities e USING connector_id_renames r
    WHERE e.entity_type = 'mcp_server' AND e.entity_id = r.old_id;
UPDATE access_control_rules x SET rule_value = r.new_id
    FROM connector_id_renames r
    WHERE x.rule_type IN ('connector', 'salesforce') AND x.rule_value = r.old_id;

DROP TABLE connector_id_renames;
