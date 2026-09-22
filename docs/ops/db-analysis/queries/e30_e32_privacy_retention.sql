-- E30: deleted users' residue; E31: retention ages; E32: secret-shaped values (counts only)
\echo == E30 user status
SELECT status, count(*) FROM users GROUP BY 1;
\echo == E30 rows keyed to a user id that no longer exists in users (per table, excluding the 'system' sentinel)
DO $$ DECLARE r record; n bigint; BEGIN
  FOR r IN SELECT table_name FROM information_schema.columns WHERE table_schema='public' AND column_name='user_id' AND table_name NOT LIKE 'analytics_report%' AND table_name NOT LIKE 'eval\_%' AND table_name NOT LIKE 'knowledge\_%' AND table_name NOT IN ('users') AND table_name IN (SELECT tablename FROM pg_tables WHERE schemaname='public') ORDER BY 1 LOOP
    EXECUTE format('SELECT count(*) FROM %I x WHERE x.user_id IS NOT NULL AND x.user_id::text <> ''system'' AND NOT EXISTS (SELECT 1 FROM users u WHERE u.id = x.user_id::text)', r.table_name) INTO n;
    IF n > 0 THEN RAISE NOTICE '%: % orphan user rows', r.table_name, n; END IF;
  END LOOP; END $$;
\echo == E31 oldest row per retention-relevant table (vs intended: logs 14/30d, ai_requests 90d, sessions expire)
SELECT 'logs' t, min(timestamp)::date oldest, max(timestamp)::date newest, count(*) FROM logs
UNION ALL SELECT 'ai_requests', min(created_at)::date, max(created_at)::date, count(*) FROM ai_requests
UNION ALL SELECT 'ai_request_messages', min(created_at)::date, max(created_at)::date, count(*) FROM ai_request_messages
UNION ALL SELECT 'analytics_events', min(timestamp)::date, max(timestamp)::date, count(*) FROM analytics_events
UNION ALL SELECT 'user_sessions', min(started_at)::date, max(started_at)::date, count(*) FROM user_sessions
UNION ALL SELECT 'governance_decisions', min(created_at)::date, max(created_at)::date, count(*) FROM governance_decisions
UNION ALL SELECT 'mcp_tool_executions', min(started_at)::date, max(started_at)::date, count(*) FROM mcp_tool_executions
UNION ALL SELECT 'plugin_usage_events', min(created_at)::date, max(created_at)::date, count(*) FROM plugin_usage_events
UNION ALL SELECT 'event_outbox', min(created_at)::date, max(created_at)::date, count(*) FROM event_outbox
UNION ALL SELECT 'engagement_events', min(created_at)::date, max(created_at)::date, count(*) FROM engagement_events
UNION ALL SELECT 'secret_audit_log', min(created_at)::date, max(created_at)::date, count(*) FROM secret_audit_log
UNION ALL SELECT 'oauth_jti_revocations', min(revoked_at)::date, max(revoked_at)::date, count(*) FROM oauth_jti_revocations
UNION ALL SELECT 'webauthn_challenges', min(created_at)::date, max(created_at)::date, count(*) FROM webauthn_challenges
UNION ALL SELECT 'bridge_exchange_codes', min(created_at)::date, max(created_at)::date, count(*) FROM bridge_exchange_codes
UNION ALL SELECT 'dev_login_codes', min(created_at)::date, max(created_at)::date, count(*) FROM dev_login_codes
UNION ALL SELECT 'id_jag_replay', min(seen_at)::date, max(seen_at)::date, count(*) FROM id_jag_replay
UNION ALL SELECT 'mcp_connector_oauth_states', min(expires_at)::date, max(expires_at)::date, count(*) FROM mcp_connector_oauth_states
UNION ALL SELECT 'ai_gateway_thought_signatures', min(expires_at)::date, max(expires_at)::date, count(*) FROM ai_gateway_thought_signatures
UNION ALL SELECT 'mcp_sessions', min(created_at)::date, max(created_at)::date, count(*) FROM mcp_sessions
UNION ALL SELECT 'mcp_proxy_identities', min(created_at)::date, max(created_at)::date, count(*) FROM mcp_proxy_identities
UNION ALL SELECT 'ai_quota_buckets', min(window_start)::date, max(window_start)::date, count(*) FROM ai_quota_buckets
UNION ALL SELECT 'user_rate_limit_buckets', min(window_start)::date, max(window_start)::date, count(*) FROM user_rate_limit_buckets;
\echo == E31 expired-but-present rows in TTL tables
SELECT (SELECT count(*) FROM webauthn_challenges WHERE expires_at < now()) webauthn_expired,
       (SELECT count(*) FROM bridge_exchange_codes WHERE expires_at < now() OR consumed_at IS NOT NULL) bridge_codes_dead,
       (SELECT count(*) FROM dev_login_codes WHERE expires_at < now() OR consumed_at IS NOT NULL) dev_codes_dead,
       (SELECT count(*) FROM id_jag_replay WHERE expires_at < now()) jag_expired,
       (SELECT count(*) FROM mcp_connector_oauth_states WHERE expires_at < now()) oauth_states_expired,
       (SELECT count(*) FROM ai_gateway_thought_signatures WHERE expires_at < now()) thought_sigs_expired,
       (SELECT count(*) FROM oauth_jti_revocations WHERE exp < now()) jti_expired,
       (SELECT count(*) FROM mcp_sessions WHERE expires_at < now() AND status='active') mcp_sessions_expired_active,
       (SELECT count(*) FROM mcp_proxy_identities WHERE expires_at < now()) proxy_identities_expired;
\echo == E32 secret-shaped values in text/json columns (counts only; excludes the encrypted-by-design columns)
SELECT 'ai_request_payloads.request_body' col, count(*) FILTER (WHERE request_body::text ~ 'sk-ant-|sk-[A-Za-z0-9]{20}') sk_keys, count(*) FILTER (WHERE request_body::text ~ 'Bearer [A-Za-z0-9._-]{20}') bearer, count(*) FILTER (WHERE request_body::text ~ 'BEGIN (RSA |EC )?PRIVATE KEY') private_key, count(*) FILTER (WHERE request_body::text ~ 'eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.') jwt FROM ai_request_payloads
UNION ALL SELECT 'ai_request_messages.content', count(*) FILTER (WHERE content ~ 'sk-ant-|sk-[A-Za-z0-9]{20}'), count(*) FILTER (WHERE content ~ 'Bearer [A-Za-z0-9._-]{20}'), count(*) FILTER (WHERE content ~ 'BEGIN (RSA |EC )?PRIVATE KEY'), count(*) FILTER (WHERE content ~ 'eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.') FROM ai_request_messages
UNION ALL SELECT 'mcp_tool_executions.input/output', count(*) FILTER (WHERE (input||output) ~ 'sk-ant-|sk-[A-Za-z0-9]{20}'), count(*) FILTER (WHERE (input||output) ~ 'Bearer [A-Za-z0-9._-]{20}'), count(*) FILTER (WHERE (input||output) ~ 'BEGIN (RSA |EC )?PRIVATE KEY'), count(*) FILTER (WHERE (input||output) ~ 'eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.') FROM mcp_tool_executions
UNION ALL SELECT 'ai_request_tool_calls.tool_input', count(*) FILTER (WHERE tool_input ~ 'sk-ant-|sk-[A-Za-z0-9]{20}'), count(*) FILTER (WHERE tool_input ~ 'Bearer [A-Za-z0-9._-]{20}'), count(*) FILTER (WHERE tool_input ~ 'BEGIN (RSA |EC )?PRIVATE KEY'), count(*) FILTER (WHERE tool_input ~ 'eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.') FROM ai_request_tool_calls
UNION ALL SELECT 'logs.message+metadata', count(*) FILTER (WHERE (message||metadata::text) ~ 'sk-ant-|sk-[A-Za-z0-9]{20}'), count(*) FILTER (WHERE (message||metadata::text) ~ 'Bearer [A-Za-z0-9._-]{20}'), count(*) FILTER (WHERE (message||metadata::text) ~ 'BEGIN (RSA |EC )?PRIVATE KEY'), count(*) FILTER (WHERE (message||metadata::text) ~ 'eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.') FROM logs
UNION ALL SELECT 'plugin_usage_events.metadata+prompt', count(*) FILTER (WHERE (metadata::text||coalesce(prompt_preview,'')) ~ 'sk-ant-|sk-[A-Za-z0-9]{20}'), count(*) FILTER (WHERE (metadata::text||coalesce(prompt_preview,'')) ~ 'Bearer [A-Za-z0-9._-]{20}'), count(*) FILTER (WHERE (metadata::text||coalesce(prompt_preview,'')) ~ 'BEGIN (RSA |EC )?PRIVATE KEY'), count(*) FILTER (WHERE (metadata::text||coalesce(prompt_preview,'')) ~ 'eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.') FROM plugin_usage_events
UNION ALL SELECT 'governance_decisions.evaluated_rules', count(*) FILTER (WHERE evaluated_rules::text ~ 'sk-ant-|sk-[A-Za-z0-9]{20}'), count(*) FILTER (WHERE evaluated_rules::text ~ 'Bearer [A-Za-z0-9._-]{20}'), count(*) FILTER (WHERE evaluated_rules::text ~ 'BEGIN (RSA |EC )?PRIVATE KEY'), count(*) FILTER (WHERE evaluated_rules::text ~ 'eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.') FROM governance_decisions;
\echo == E32 mcp_proxy_identities.auth_token: stored in clear?
SELECT count(*), count(*) FILTER (WHERE auth_token ~ '^eyJ') jwt_shaped, count(*) FILTER (WHERE length(auth_token) > 40) long FROM mcp_proxy_identities;
