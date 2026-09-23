-- D23: user_sessions hygiene
\echo == D23 session anomalies
SELECT count(*) sessions,
  count(*) FILTER (WHERE ended_at IS NULL AND last_activity_at < now() - interval '7 days') open_idle_7d,
  count(*) FILTER (WHERE expires_at < now() AND ended_at IS NULL) expired_not_ended,
  count(*) FILTER (WHERE revoked_at IS NOT NULL) revoked,
  count(*) FILTER (WHERE is_bot) bots, count(*) FILTER (WHERE is_ai_crawler) ai_crawlers, count(*) FILTER (WHERE is_scanner) scanners, count(*) FILTER (WHERE is_behavioral_bot) behavioral_bots,
  count(*) FILTER (WHERE user_type='anon') anon, count(*) FILTER (WHERE ended_at IS NOT NULL) ended,
  count(*) FILTER (WHERE duration_seconds IS NULL) no_duration
FROM user_sessions;
\echo == D23 sessions by source / client_type
SELECT session_source, client_type, count(*), count(*) FILTER (WHERE EXISTS (SELECT 1 FROM ai_requests r WHERE r.session_id = s.session_id)) with_ai_requests FROM user_sessions s GROUP BY 1,2 ORDER BY 3 DESC;
\echo == D23 users with > 50 sessions in a day
SELECT user_id, started_at::date d, count(*) FROM user_sessions GROUP BY 1,2 HAVING count(*) > 50 ORDER BY 3 DESC;
\echo == D23 sessions per user (top 5 by count; ids only)
SELECT user_id, count(*) sessions, min(started_at)::date first, max(started_at)::date last FROM user_sessions GROUP BY 1 ORDER BY 2 DESC LIMIT 5;
\echo == D23 the 7 expired-not-ended sessions
SELECT session_id, session_source, expires_at::timestamp(0), last_activity_at::timestamp(0), request_count FROM user_sessions WHERE expires_at < now() AND ended_at IS NULL;
\echo == D23 user_contexts.session_id vs the request's session: contexts spanning sessions
SELECT count(*) contexts, count(*) FILTER (WHERE (SELECT count(DISTINCT session_id) FROM ai_requests r WHERE r.context_id = c.context_id) > 1) contexts_spanning_sessions,
       count(*) FILTER (WHERE NOT EXISTS (SELECT 1 FROM ai_requests r WHERE r.context_id = c.context_id AND r.session_id = c.session_id)) context_session_not_among_requests
FROM user_contexts c;
