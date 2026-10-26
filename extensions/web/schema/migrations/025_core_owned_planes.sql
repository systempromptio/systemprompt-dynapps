-- Access control, governance and gateway policies are core's planes: core
-- records their state in `config_plane_state` and is their only writer, so
-- `sync_state` keeps rows for the install's own planes alone and the staged
-- copy of the governance chain goes.
--
-- Why: lock footprint. `sync_state` holds one row per plane (five before,
-- two after) and the two governance_chain tables a handful of rows each;
-- the DELETE, the CHECK and the DROPs hold their locks for milliseconds.
DELETE FROM sync_state WHERE plane IN ('access_control', 'governance', 'gateway_policies');

ALTER TABLE sync_state DROP CONSTRAINT IF EXISTS sync_state_plane_check;
ALTER TABLE sync_state
    ADD CONSTRAINT sync_state_plane_check CHECK (plane IN ('groups', 'gateway_routes')) NOT VALID;
ALTER TABLE sync_state VALIDATE CONSTRAINT sync_state_plane_check;

DROP TABLE IF EXISTS governance_chain_settings;
DROP TABLE IF EXISTS governance_chain;
