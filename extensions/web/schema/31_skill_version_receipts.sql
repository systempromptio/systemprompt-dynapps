-- Which published revision a skill invocation ran.
--
-- An invocation is revision-verified when exactly one authenticated device
-- identity (30_feedback_authenticated_events.sql) matches it, and that
-- identity's bound session (the bridge's session bindings) names exactly one
-- fully verified installation receipt for a granted resource whose key is
-- the invoked skill. `analysis_skill_version_events` is every skill
-- invocation with that receipt, or `revision_unknown`. Migration 105 drops
-- the view before this file recreates it, because the `traffic_class`
-- column it once carried (always 'production') cannot be removed in place.

CREATE OR REPLACE FUNCTION feedback_event_identity(event_key TEXT)
RETURNS JSONB LANGUAGE sql STABLE AS $$
WITH identities AS (
 SELECT a.consumer_id,a.device_id,a.host,a.native_session_id
 FROM feedback_authenticated_events a JOIN plugin_usage_events e ON e.id=a.event_id
 WHERE a.event_id=event_key AND a.consumer_id=e.user_id AND a.native_session_id=e.session_id
)
SELECT CASE WHEN count(*)=1 THEN (jsonb_agg(jsonb_build_object('status','authenticated','consumer_id',consumer_id,'device_id',device_id,'host',host,'session_id',native_session_id)))->0
 ELSE jsonb_build_object('status','historical_unknown') END FROM identities
$$;

CREATE OR REPLACE FUNCTION feedback_event_receipt(event_key TEXT)
RETURNS JSONB LANGUAGE sql STABLE AS $$
WITH identity AS (SELECT feedback_event_identity(event_key) AS value), matches AS (
 SELECT DISTINCT r.id,r.installation_id,r.generation,r.resource_id,p.revision_id
 FROM analysis_skill_events e CROSS JOIN identity i
 JOIN managed_consumer_session_bindings b ON b.consumer_id=i.value->>'consumer_id'
 AND b.device_id=i.value->>'device_id' AND b.host=i.value->>'host' AND b.native_session_id=i.value->>'session_id'
 JOIN managed_installation_receipts r ON r.id=b.receipt_id AND r.consumer_id=b.consumer_id AND r.device_id=b.device_id AND r.host=b.host
 JOIN managed_publications p ON p.id=r.publication_id AND p.owner_id=r.owner_id
 JOIN managed_resources m ON m.id=r.resource_id AND m.owner_id=r.owner_id
 JOIN managed_consumer_grants g ON g.resource_id=r.resource_id AND g.owner_id=r.owner_id AND g.consumer_id=r.consumer_id AND g.revoked_at IS NULL
 WHERE e.id=event_key AND r.fully_verified AND jsonb_array_length(COALESCE(r.consumer_evidence->'runtime_files','[]'::jsonb))>0
 AND replace(m.resource_key,'-','_')=replace(split_part(e.skill,':',2),'-','_')
)
SELECT CASE WHEN count(*)=1 THEN (jsonb_agg(jsonb_build_object('receipt_id',id,'installation_id',installation_id,'generation',generation,'resource_id',resource_id,'revision_id',revision_id)))->0
 ELSE NULL END FROM matches
$$;

CREATE OR REPLACE VIEW analysis_skill_version_events AS
SELECT e.id AS invocation_id,e.user_id,e.session_id,e.plugin_id,e.skill,
       e.tool_use_id,e.source,e.invoked_at,a.value->>'installation_id' AS installation_id,
       a.value->>'resource_id' AS resource_id,a.value->>'revision_id' AS revision_id,
       (a.value->>'generation')::bigint AS publication_generation,
       CASE WHEN a.value IS NOT NULL THEN 'verified' ELSE 'revision_unknown' END AS attribution_status
FROM analysis_skill_events e
-- OFFSET 0 fences the LATERAL against pull-up. Without it Postgres inlines the
-- subquery and re-evaluates this six-join function once per referenced column,
-- so selecting the view cost five calls per row instead of one.
CROSS JOIN LATERAL (SELECT feedback_event_receipt(e.id) AS value OFFSET 0) a;
