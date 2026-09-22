WITH classified AS MATERIALIZED (
            SELECT cr.* FROM conversation_requests cr
            WHERE cr.context_id IN (
                SELECT DISTINCT context_id FROM ai_requests
                WHERE ($1::text[] IS NULL OR user_id = ANY($1)) AND context_id <> $4
            )
        ),
        gw AS (
            SELECT
                context_id,
                MAX(client_session_id)                      AS client_session_id,
                COUNT(*) FILTER (WHERE effective_kind = 'turn')::bigint  AS turns,
                COUNT(*) FILTER (WHERE effective_kind <> 'turn')::bigint AS side_calls,
                COALESCE(SUM(input_tokens), 0)::bigint      AS tokens_in,
                COALESCE(SUM(output_tokens), 0)::bigint     AS tokens_out,
                COALESCE(SUM(cost_microdollars), 0)::bigint AS cost_microdollars,
                MIN(created_at)                             AS started_at,
                MAX(created_at)                             AS last_at
            FROM classified
            WHERE context_id <> $4
              AND ($1::text[] IS NULL OR user_id = ANY($1))
            GROUP BY context_id
        ),
        gw_latest AS (
            SELECT DISTINCT ON (context_id) context_id, user_id, model, client_kind, client_attestation
            FROM classified
            WHERE context_id <> $4
            ORDER BY context_id, created_at DESC
        ),
        conversations AS (
            SELECT
                g.context_id           AS context_id,
                g.client_session_id    AS session_id,
                l.user_id              AS user_id,
                COALESCE(pss.ai_title, NULLIF(uc.name, 'Gateway conversation')) AS title,
                CASE WHEN $5::text IS NOT NULL THEN conversation_opening_prompt(g.context_id, 200) END AS preview,
                l.model                AS model,
                l.client_kind          AS client_kind,
                l.client_attestation   AS client_attestation,
                g.started_at           AS started_at,
                g.last_at              AS last_at,
                g.turns                AS turns,
                g.tokens_in            AS tokens_in,
                g.tokens_out           AS tokens_out,
                g.cost_microdollars    AS cost_microdollars,
                g.side_calls           AS side_calls
            FROM gw g
            JOIN gw_latest l ON l.context_id = g.context_id
            LEFT JOIN user_contexts uc ON uc.context_id = g.context_id
            LEFT JOIN plugin_session_summaries pss ON pss.session_id = g.client_session_id
            WHERE ($6::bool OR g.turns > 0)
              AND ($5::text IS NULL
                   OR pss.ai_title ILIKE $5
                   OR uc.name ILIKE $5
                   OR conversation_opening_prompt(g.context_id, 200) ILIKE $5
                   OR g.context_id ILIKE $5
                   OR l.model ILIKE $5)
        )
        , counted AS MATERIALIZED (
            SELECT *, COUNT(*) OVER ()::bigint AS total_count FROM conversations
        ), page AS MATERIALIZED (
            SELECT * FROM counted
            ORDER BY last_at DESC, context_id
            LIMIT $2 OFFSET $3
        )
        , enriched AS (SELECT
            context_id                      AS context_id,
            session_id                      AS session_id,
            COALESCE(page.user_id, '')      AS user_id,
            -- Why: a raw uuid names nobody. The join runs after LIMIT/OFFSET,
            -- so it costs one lookup per rendered row, not per conversation.
            COALESCE(NULLIF(u.display_name, ''), NULLIF(u.email, ''),
                     COALESCE(page.user_id, '')) AS user_label,
            title                           AS title,
            COALESCE(preview, conversation_opening_prompt(context_id, 200)) AS preview,
            model                           AS model,
            client_kind                     AS client_kind,
            client_attestation              AS client_attestation,
            started_at                      AS started_at,
            last_at                         AS last_at,
            turns                           AS turns,
            tokens_in                       AS total_input_tokens,
            tokens_out                      AS total_output_tokens,
            cost_microdollars               AS cost_microdollars,
            side_calls                      AS side_call_count,
            total_count AS total_count
        FROM page LEFT JOIN users u ON u.id = page.user_id
        ORDER BY last_at DESC, context_id
        ) SELECT jsonb_build_object(
            'items', COALESCE((SELECT jsonb_agg(to_jsonb(e) ORDER BY last_at DESC, context_id) FROM enriched e), '[]'::jsonb),
            'total', (SELECT COUNT(*) FROM counted)
        ) AS "payload!: Json<HistoryPageResult>",
          ARRAY(SELECT context_id FROM page) AS "context_ids!: Vec<ContextId>"
