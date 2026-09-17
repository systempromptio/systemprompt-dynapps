-- One statement for the classified-conversation page: the filtered set once,
-- then its totals, its breakdown for the chosen dimension, and one page of
-- rows. Classification predicates narrow `conversation_analyses` first so
-- conversation_metrics_for only aggregates the contexts that can appear.
WITH picked AS MATERIALIZED (
    SELECT a.context_id, a.category, a.summary, a.tags, a.skills_used, a.outcome,
           a.confidence, a.classified_at, a.model AS judge_model
    FROM conversation_analyses a
    WHERE a.status = 'classified'
      AND ($1::text[] IS NULL OR a.user_id = ANY($1))
      AND ($2::text IS NULL OR a.user_id = $2)
      AND ($3::text IS NULL OR a.category = $3)
      AND ($4::text IS NULL OR a.outcome = $4)
), ids AS MATERIALIZED (
    SELECT COALESCE(array_agg(context_id), ARRAY[]::text[]) AS ids FROM picked
), f AS MATERIALIZED (
    SELECT r.*, p.category, p.summary, p.tags, p.skills_used, p.outcome, p.confidence,
           p.classified_at, p.judge_model,
           r.total_input_tokens + r.total_output_tokens AS total_tokens,
           sc.intent AS intent_score, sc.quality AS quality_score,
           sc.speed AS speed_score, sc.cost AS cost_score,
           COALESCE(sc.scored_criteria, 0)::int AS scored_criteria,
           COALESCE((SELECT array_agg(s.skill ORDER BY s.first_at) FROM conversation_skill_uses s
                     WHERE s.client_session_id = r.client_session_id), ARRAY[]::text[]) AS skills_hooked
    FROM ids CROSS JOIN LATERAL conversation_metrics_for(ids.ids) r
    JOIN picked p ON p.context_id = r.context_id
    LEFT JOIN evaluation_session_scorecard sc ON sc.session_id = r.client_session_id
    WHERE ($1::text[] IS NULL OR r.user_id = ANY($1))
      AND ($2::text IS NULL OR r.user_id = $2)
      AND ($5::text IS NULL OR $5 = ANY(p.skills_used)
           OR EXISTS (SELECT 1 FROM conversation_skill_uses s
                      WHERE s.client_session_id = r.client_session_id AND s.skill = $5))
      AND ($6::text IS NULL OR r.display_name ILIKE $6 OR r.user_id ILIKE $6 OR p.summary ILIKE $6
           OR array_to_string(p.tags, ' ') ILIKE $6
           OR conversation_title(r.context_id, r.client_session_id) ILIKE $6)
      AND ($7::timestamptz IS NULL OR r.last_at >= $7)
      AND ($8::timestamptz IS NULL OR r.last_at < $8)
      AND ($14::text IS NULL
           OR ($14 = 'scored' AND sc.scored_criteria > 0)
           OR ($14 = 'unscored' AND sc.scored_criteria IS NULL)
           OR ($14 = 'low' AND sc.quality < 50))
), totals AS (
    SELECT COUNT(*)::bigint AS conversations, COUNT(DISTINCT user_id)::bigint AS users,
           COALESCE(SUM(turn_count), 0)::bigint AS turns,
           COALESCE(SUM(tool_call_count), 0)::bigint AS tool_calls,
           COALESCE(SUM(total_tokens), 0)::bigint AS total_tokens,
           COALESCE(SUM(total_cost_microdollars), 0)::bigint AS total_cost_microdollars,
           COUNT(*) FILTER (WHERE outcome = 'achieved')::bigint AS achieved,
           COUNT(*) FILTER (WHERE scored_criteria > 0)::bigint AS scored,
           AVG(quality_score)::float8 AS quality_avg,
           AVG(intent_score)::float8 AS intent_avg,
           AVG(speed_score)::float8 AS speed_avg,
           AVG(cost_score)::float8 AS cost_avg,
           (SELECT COUNT(*)::bigint FROM conversation_analyses u
             WHERE u.status <> 'classified'
               AND ($1::text[] IS NULL OR u.user_id = ANY($1))
               AND ($2::text IS NULL OR u.user_id = $2)) AS unclassified
    FROM f
), keyed AS (
    SELECT f.category, f.outcome, f.user_id, f.turn_count, f.total_cost_microdollars,
           f.quality_score, f.scored_criteria,
           CASE $13::text
               WHEN 'group' THEN COALESCE(f.group_name, 'Unattributed')
               WHEN 'project' THEN COALESCE(f.project_name, 'Unattributed')
               WHEN 'user' THEN COALESCE(f.display_name, f.user_id, 'Unattributed')
               WHEN 'outcome' THEN COALESCE(f.outcome, 'unclear')
               ELSE f.category END AS bucket,
           CASE $13::text WHEN 'user' THEN f.user_id ELSE NULL END AS bucket_user
    FROM f
), breakdown AS (
    SELECT k.bucket AS label, k.bucket_user AS user_id,
           COUNT(*)::bigint AS conversations, COUNT(DISTINCT k.user_id)::bigint AS users,
           COALESCE(SUM(k.turn_count), 0)::bigint AS turns,
           COALESCE(SUM(k.total_cost_microdollars), 0)::bigint AS total_cost_microdollars,
           COUNT(*) FILTER (WHERE k.outcome = 'achieved')::bigint AS achieved,
           COUNT(*) FILTER (WHERE k.scored_criteria > 0)::bigint AS scored,
           AVG(k.quality_score)::float8 AS quality_avg,
           mode() WITHIN GROUP (ORDER BY k.category) AS top_category
    FROM keyed k
    GROUP BY k.bucket, k.bucket_user
    ORDER BY conversations DESC, label
    LIMIT 100
), selected AS MATERIALIZED (
    SELECT f.*, conversation_title(f.context_id, f.client_session_id) AS title,
           ROW_NUMBER() OVER (ORDER BY
             CASE WHEN $9 = 'activity' AND $10::boolean THEN f.last_at END DESC NULLS LAST,
             CASE WHEN $9 = 'activity' AND NOT $10::boolean THEN f.last_at END ASC NULLS LAST,
             CASE WHEN $9 = 'turns' AND $10::boolean THEN f.turn_count END DESC,
             CASE WHEN $9 = 'turns' AND NOT $10::boolean THEN f.turn_count END ASC,
             CASE WHEN $9 = 'tokens' AND $10::boolean THEN f.total_tokens END DESC,
             CASE WHEN $9 = 'tokens' AND NOT $10::boolean THEN f.total_tokens END ASC,
             CASE WHEN $9 = 'cost' AND $10::boolean THEN f.total_cost_microdollars END DESC,
             CASE WHEN $9 = 'cost' AND NOT $10::boolean THEN f.total_cost_microdollars END ASC,
             CASE WHEN $9 = 'quality' AND $10::boolean THEN f.quality_score END DESC NULLS LAST,
             CASE WHEN $9 = 'quality' AND NOT $10::boolean THEN f.quality_score END ASC NULLS LAST,
             f.context_id) AS position
    FROM f
    ORDER BY position
    LIMIT $11 OFFSET $12
), skills AS (
    SELECT skill, COUNT(*)::bigint AS conversations FROM (
        SELECT f.context_id, unnest(f.skills_used || f.skills_hooked) AS skill FROM f
    ) s GROUP BY skill ORDER BY conversations DESC, skill LIMIT 200
), users AS (
    SELECT f.user_id, MAX(f.display_name) AS display_name, COUNT(*)::bigint AS conversations
    FROM f WHERE f.user_id IS NOT NULL GROUP BY f.user_id ORDER BY conversations DESC LIMIT 500
)
SELECT jsonb_build_object(
    'rows', COALESCE((SELECT jsonb_agg(to_jsonb(s) ORDER BY s.position) FROM selected s), '[]'::jsonb),
    'totals', (SELECT to_jsonb(t) FROM totals t),
    'breakdown', COALESCE((SELECT jsonb_agg(to_jsonb(b)) FROM breakdown b), '[]'::jsonb),
    'skills', COALESCE((SELECT jsonb_agg(to_jsonb(k)) FROM skills k), '[]'::jsonb),
    'users', COALESCE((SELECT jsonb_agg(to_jsonb(u)) FROM users u), '[]'::jsonb)
) AS "payload!: Json<ConversationAnalysisWire>",
    ARRAY(SELECT context_id FROM selected ORDER BY position) AS "context_ids!: Vec<ContextId>"
