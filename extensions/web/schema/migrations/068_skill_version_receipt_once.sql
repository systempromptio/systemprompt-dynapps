-- The view resolves a skill event's installation receipt through a LATERAL
-- subquery and then reads five columns out of it. Postgres pulls that subquery
-- up into the select list, so `feedback_event_receipt` -- a six-join SQL
-- function -- ran once per referenced column rather than once per row. Selecting
-- the whole view therefore cost five function calls per event: 1179 ms for 771
-- rows on a local database, and the Impact page issues three such queries.
--
-- OFFSET 0 is an optimisation fence. It blocks the pull-up without changing the
-- result, so the function runs exactly once per row. The same measurement drops
-- to 239 ms; the Impact page's three queries go from 3.65 s to 1.15 s.
--
-- The column list is unchanged, so CREATE OR REPLACE is enough and this
-- migration is idempotent.
CREATE OR REPLACE VIEW analysis_skill_version_events AS
SELECT e.id AS invocation_id,e.user_id,e.session_id,e.plugin_id,e.skill,
       e.tool_use_id,e.source,e.invoked_at,a.value->>'installation_id' AS installation_id,
       a.value->>'resource_id' AS resource_id,a.value->>'revision_id' AS revision_id,
       (a.value->>'generation')::bigint AS publication_generation,'production'::text AS traffic_class,
       CASE WHEN a.value IS NOT NULL THEN 'verified' ELSE 'revision_unknown' END AS attribution_status
FROM analysis_skill_events e
CROSS JOIN LATERAL (SELECT feedback_event_receipt(e.id) AS value OFFSET 0) a;
