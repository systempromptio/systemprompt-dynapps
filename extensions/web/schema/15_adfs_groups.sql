-- Web-owned AD group membership side table.
--
-- One row per (user, AD group) the ADFS id_token carried at the user's most
-- recent sign-in, restricted to the groups `adfs.yaml` maps. Written by the
-- SSO callback (replace, not union — a group the user has left is gone at
-- their next login) and read by the `adfs_group` subject dimension so
-- `access_control_rules` rows at `rule_type = 'adfs_group'` bind at
-- enforcement time.
--
-- Cascades on delete, mirroring `user_profile_ext` in 13_web_side_tables.sql —
-- the web extension owns its own table rather than ALTERing the vendored
-- `federated_identities` table (which stores only the OIDC `sub`).

CREATE TABLE IF NOT EXISTS adfs_group_memberships (
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    group_name TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, group_name)
);
