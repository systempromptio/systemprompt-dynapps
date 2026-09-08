-- One snapshot and one numeric aggregation for rows, KPIs and pagination.
-- Date/identity predicates select candidate contexts, never truncate their turns.
WITH candidates AS MATERIALIZED (
    SELECT COALESCE(array_agg(DISTINCT ar.context_id::text), ARRAY[]::text[]) AS ids
    FROM ai_requests ar
    LEFT JOIN user_contexts c ON c.context_id = ar.context_id
    WHERE ar.context_id <> '00000000-0000-0000-0000-4c4547414359'
      AND ($1::text IS NULL OR ar.user_id = $1)
      AND ($2::text[] IS NULL OR ar.user_id = ANY($2))
      AND ($5::timestamptz IS NULL OR ar.created_at >= $5 OR c.updated_at >= $5)
), f AS MATERIALIZED (
    SELECT r.*, r.total_input_tokens + r.total_output_tokens AS total_tokens
    FROM candidates c CROSS JOIN LATERAL conversation_metrics_for(c.ids) r
    WHERE ($1::text IS NULL OR r.user_id = $1)
      AND ($2::text[] IS NULL OR r.user_id = ANY($2))
      AND ($3::text IS NULL OR r.model = $3)
      AND ($4::text IS NULL OR r.display_name ILIKE $4 OR r.context_id ILIKE $4
           OR r.user_id ILIKE $4 OR conversation_title(r.context_id, r.client_session_id) ILIKE $4)
      AND ($5::timestamptz IS NULL OR r.last_at >= $5)
      AND ($6::timestamptz IS NULL OR r.last_at < $6)
      AND ($7::boolean OR r.turn_count > 0)
      AND (NOT $8::boolean OR r.error_count > 0)
), totals AS (
    SELECT COUNT(*)::bigint AS conversations, COUNT(DISTINCT user_id)::bigint AS users,
           COALESCE(SUM(turn_count), 0)::bigint AS turns,
           COALESCE(SUM(side_call_count), 0)::bigint AS side_calls,
           COALESCE(SUM(tool_call_count), 0)::bigint AS tool_calls,
           COUNT(*) FILTER (WHERE error_count > 0)::bigint AS error_conversations,
           COALESCE(SUM(total_tokens), 0)::bigint AS total_tokens,
           COALESCE(SUM(total_cost_microdollars), 0)::bigint AS total_cost_microdollars,
           COALESCE(SUM(side_call_cost_microdollars), 0)::bigint AS side_call_cost_microdollars
    FROM f
), user_page AS MATERIALIZED (
    SELECT r.*, ROW_NUMBER() OVER (ORDER BY CASE WHEN $9 = 'activity' AND $10::boolean THEN r.last_at END DESC NULLS LAST,
        CASE WHEN $9 = 'activity' AND NOT $10::boolean THEN r.last_at END ASC NULLS LAST,
        CASE WHEN $9 = 'turns' AND $10::boolean THEN r.turn_count END DESC,
        CASE WHEN $9 = 'turns' AND NOT $10::boolean THEN r.turn_count END ASC,
        CASE WHEN $9 = 'tokens' AND $10::boolean THEN r.total_tokens END DESC,
        CASE WHEN $9 = 'tokens' AND NOT $10::boolean THEN r.total_tokens END ASC,
        CASE WHEN $9 = 'cost' AND $10::boolean THEN r.total_cost_microdollars END DESC,
        CASE WHEN $9 = 'cost' AND NOT $10::boolean THEN r.total_cost_microdollars END ASC, r.user_id) AS position
    FROM (
        SELECT user_id, MAX(display_name) AS display_name,
               COUNT(*)::bigint AS conversation_count, SUM(turn_count)::bigint AS turn_count,
               SUM(side_call_count)::bigint AS side_call_count,
               SUM(total_tokens)::bigint AS total_tokens,
               SUM(total_cost_microdollars)::bigint AS total_cost_microdollars, MAX(last_at) AS last_at
        FROM f WHERE $13::boolean AND NOT $14::boolean AND user_id IS NOT NULL GROUP BY user_id
    ) r
    ORDER BY CASE WHEN $9 = 'activity' AND $10::boolean THEN r.last_at END DESC NULLS LAST,
        CASE WHEN $9 = 'activity' AND NOT $10::boolean THEN r.last_at END ASC NULLS LAST,
        CASE WHEN $9 = 'turns' AND $10::boolean THEN r.turn_count END DESC,
        CASE WHEN $9 = 'turns' AND NOT $10::boolean THEN r.turn_count END ASC,
        CASE WHEN $9 = 'tokens' AND $10::boolean THEN r.total_tokens END DESC,
        CASE WHEN $9 = 'tokens' AND NOT $10::boolean THEN r.total_tokens END ASC,
        CASE WHEN $9 = 'cost' AND $10::boolean THEN r.total_cost_microdollars END DESC,
        CASE WHEN $9 = 'cost' AND NOT $10::boolean THEN r.total_cost_microdollars END ASC, r.user_id LIMIT $11 OFFSET $12
), selected AS MATERIALIZED (
    (SELECT r.*, ROW_NUMBER() OVER (ORDER BY CASE WHEN $9 = 'activity' AND $10::boolean THEN r.last_at END DESC NULLS LAST,
        CASE WHEN $9 = 'activity' AND NOT $10::boolean THEN r.last_at END ASC NULLS LAST,
        CASE WHEN $9 = 'turns' AND $10::boolean THEN r.turn_count END DESC,
        CASE WHEN $9 = 'turns' AND NOT $10::boolean THEN r.turn_count END ASC,
        CASE WHEN $9 = 'tokens' AND $10::boolean THEN r.total_tokens END DESC,
        CASE WHEN $9 = 'tokens' AND NOT $10::boolean THEN r.total_tokens END ASC,
        CASE WHEN $9 = 'cost' AND $10::boolean THEN r.total_cost_microdollars END DESC,
        CASE WHEN $9 = 'cost' AND NOT $10::boolean THEN r.total_cost_microdollars END ASC, r.context_id) AS position
     FROM f r WHERE NOT $13::boolean AND NOT $14::boolean
     ORDER BY CASE WHEN $9 = 'activity' AND $10::boolean THEN r.last_at END DESC NULLS LAST,
        CASE WHEN $9 = 'activity' AND NOT $10::boolean THEN r.last_at END ASC NULLS LAST,
        CASE WHEN $9 = 'turns' AND $10::boolean THEN r.turn_count END DESC,
        CASE WHEN $9 = 'turns' AND NOT $10::boolean THEN r.turn_count END ASC,
        CASE WHEN $9 = 'tokens' AND $10::boolean THEN r.total_tokens END DESC,
        CASE WHEN $9 = 'tokens' AND NOT $10::boolean THEN r.total_tokens END ASC,
        CASE WHEN $9 = 'cost' AND $10::boolean THEN r.total_cost_microdollars END DESC,
        CASE WHEN $9 = 'cost' AND NOT $10::boolean THEN r.total_cost_microdollars END ASC, r.context_id LIMIT $11 OFFSET $12)
    UNION ALL
    SELECT preview.*, u.position FROM user_page u
    CROSS JOIN LATERAL (
        SELECT r.* FROM f r WHERE r.user_id = u.user_id
        ORDER BY r.last_at DESC NULLS LAST, r.context_id LIMIT 3
    ) preview
), enriched AS MATERIALIZED (
    SELECT s.*, conversation_title(s.context_id, s.client_session_id) AS title FROM selected s
), summaries AS (
    SELECT u.*, COALESCE((
        SELECT jsonb_agg(DISTINCT ar.model ORDER BY ar.model)
        FROM ai_requests ar JOIN f ON f.context_id = ar.context_id
        WHERE ar.user_id = u.user_id AND ar.model IS NOT NULL
    ), '[]'::jsonb) AS models,
    (SELECT to_jsonb(e) FROM enriched e WHERE e.user_id = u.user_id
     ORDER BY e.last_at DESC NULLS LAST, e.context_id LIMIT 1) AS latest
    FROM user_page u
)
SELECT jsonb_build_object(
    'conversations', COALESCE((SELECT jsonb_agg(to_jsonb(e) ORDER BY e.position, e.last_at DESC NULLS LAST, e.context_id) FROM enriched e), '[]'::jsonb),
    'user_summaries', COALESCE((SELECT jsonb_agg(to_jsonb(s) ORDER BY s.position) FROM summaries s), '[]'::jsonb),
    'totals', (SELECT to_jsonb(t) FROM totals t)
) AS "payload!: Json<ConversationPageResult>"
