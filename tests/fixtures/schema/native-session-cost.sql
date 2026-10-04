-- Isolated migration benchmark: 10,000 audit requests in 1,000 native sessions.
-- Run only in a disposable database restored from release-baseline-0.62.0.sql.
INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
VALUES ('native-cost-user', 'native-cost-user', 'native-cost@example.test',
        'Native cost fixture', 'active', true, ARRAY['user']);
CREATE TEMP TABLE benchmark_contexts AS
SELECT n, gen_random_uuid()::text AS context_id, gen_random_uuid()::text AS session_id
FROM generate_series(1, 1000) n;
INSERT INTO ai_requests
    (id, request_id, user_id, context_id, client_session_id, provider, model,
     input_tokens, output_tokens, tokens_used, cost_microdollars, latency_ms,
     status, actor_kind, actor_id, created_at, updated_at)
SELECT id, id, 'native-cost-user', context_id, session_id, 'anthropic', 'claude-test',
       100, 20, 120, 5000, 250, 'completed', 'user', 'native-cost-user', now(), now()
FROM benchmark_contexts CROSS JOIN generate_series(1, 10) request
CROSS JOIN LATERAL (SELECT gen_random_uuid()::text AS id WHERE request > 0 AND n > 0) ids;
SELECT refresh_conversation_facts(ARRAY(SELECT context_id FROM benchmark_contexts));
