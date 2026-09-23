-- There is one Salesforce org, ontrq.my.salesforce.com, not the three the
-- connector was built for, and one connector onto it rather than six. The
-- US/UK/dev split was never real: four of the six registry entries still
-- carried REPLACE- placeholder domains and could not be connected, yet each
-- rendered its own "Not configured" card. All six ids collapse onto `salesforce`,
-- on the Headless 360 endpoint the sales skills call -- the same shape as the
-- `atlassian` and `google_workspace` connectors.
--
-- Grants are removed rather than renamed, for the reason migration 088 gave:
-- the sealed grant binds its provider inside the ciphertext, so a renamed grant
-- fails the owner check on every call. Here they could not be renamed anyway --
-- six ids mapping onto one would put several rows on one user, and a token
-- minted against the dev org is not a token for ontrq. Each user connects once
-- more.
--
-- The surviving id is the one the Headless 360 grant already used, so that
-- entry carries its access rows across; every other id is retired outright.

CREATE TEMP TABLE salesforce_id_collapse (old_id TEXT PRIMARY KEY, new_id TEXT);
INSERT INTO salesforce_id_collapse VALUES
    ('salesforce-headless-us', 'salesforce'),
    -- No successor: these are retired, not renamed.
    ('salesforce-crm-us', NULL),
    ('salesforce-crm-uk', NULL),
    ('salesforce-crm-dev', NULL),
    ('salesforce-headless-uk', NULL),
    ('salesforce-headless-dev', NULL);

DELETE FROM mcp_connector_credentials c USING salesforce_id_collapse r WHERE c.provider = r.old_id;
DELETE FROM mcp_connector_accounts a USING salesforce_id_collapse r WHERE a.provider = r.old_id;
DELETE FROM mcp_connector_oauth_states s USING salesforce_id_collapse r WHERE s.provider = r.old_id;
-- Identities map a person to a username in one org. The org is gone, so the
-- mapping is too; the surviving pair keeps its rows under the new ids.
DELETE FROM salesforce_user_identities i USING salesforce_id_collapse r
    WHERE i.provider = r.old_id AND r.new_id IS NULL;
UPDATE salesforce_user_identities i SET provider = r.new_id
    FROM salesforce_id_collapse r WHERE i.provider = r.old_id AND r.new_id IS NOT NULL;

-- The rules' entity FK is neither deferrable nor ON UPDATE CASCADE, so the
-- entity row is copied under its new id, the rules move across, and the old
-- row goes last (its cascade then has nothing left to delete).
INSERT INTO access_control_entities (entity_type, entity_id, default_included, source, created_at, updated_at)
    SELECT e.entity_type, r.new_id, e.default_included, e.source, e.created_at, now()
    FROM access_control_entities e JOIN salesforce_id_collapse r
        ON e.entity_type = 'mcp_server' AND e.entity_id = r.old_id
    WHERE r.new_id IS NOT NULL
    ON CONFLICT (entity_type, entity_id) DO NOTHING;
-- Why DELETE before UPDATE: UNIQUE(entity_type, entity_id, rule_type, rule_value)
-- means a rule that already exists on the target id would collide with the one
-- moving onto it. The duplicate is dropped first, so the move cannot fail.
DELETE FROM access_control_rules x USING salesforce_id_collapse r
    WHERE x.entity_type = 'mcp_server' AND x.entity_id = r.old_id AND r.new_id IS NOT NULL
      AND EXISTS (
          SELECT 1 FROM access_control_rules t
          WHERE t.entity_type = 'mcp_server' AND t.entity_id = r.new_id
            AND t.rule_type = x.rule_type AND t.rule_value = x.rule_value);
UPDATE access_control_rules x SET entity_id = r.new_id
    FROM salesforce_id_collapse r
    WHERE x.entity_type = 'mcp_server' AND x.entity_id = r.old_id AND r.new_id IS NOT NULL;
-- Cascades the rules of the four retired orgs with them.
DELETE FROM access_control_entities e USING salesforce_id_collapse r
    WHERE e.entity_type = 'mcp_server' AND e.entity_id = r.old_id;

-- The connector and salesforce bands name a server id as their VALUE, on any
-- entity -- the marketplace band is the one that matters here. Retired ids
-- leave no band behind; renamed ones follow.
DELETE FROM access_control_rules x USING salesforce_id_collapse r
    WHERE x.rule_type IN ('connector', 'salesforce') AND x.rule_value = r.old_id
      AND r.new_id IS NULL;
DELETE FROM access_control_rules x USING salesforce_id_collapse r
    WHERE x.rule_type IN ('connector', 'salesforce') AND x.rule_value = r.old_id AND r.new_id IS NOT NULL
      AND EXISTS (
          SELECT 1 FROM access_control_rules t
          WHERE t.entity_type = x.entity_type AND t.entity_id = x.entity_id
            AND t.rule_type = x.rule_type AND t.rule_value = r.new_id);
UPDATE access_control_rules x SET rule_value = r.new_id
    FROM salesforce_id_collapse r
    WHERE x.rule_type IN ('connector', 'salesforce') AND x.rule_value = r.old_id AND r.new_id IS NOT NULL;

DROP TABLE salesforce_id_collapse;
