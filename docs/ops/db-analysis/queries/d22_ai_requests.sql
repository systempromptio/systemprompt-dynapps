-- D22: ai_requests temporal/state anomalies
\echo == D22 anomalies
SELECT count(*) FILTER (WHERE completed_at < created_at) completed_before_created,
       count(*) FILTER (WHERE status='completed' AND latency_ms IS NULL) completed_no_latency,
       count(*) FILTER (WHERE status='completed' AND completed_at IS NULL) completed_no_completed_at,
       count(*) FILTER (WHERE status IN ('pending','streaming') AND created_at < now() - interval '1 hour') stuck_pending,
       count(*) FILTER (WHERE created_at > now()) in_future,
       count(*) FILTER (WHERE finish_reason IS NULL) finish_reason_null,
       count(*) FILTER (WHERE served_provider IS NULL) served_provider_null,
       count(*) FILTER (WHERE upstream_latency_ms IS NULL AND status='completed') upstream_latency_null,
       count(*) FILTER (WHERE latency_ms > 600000) latency_over_10min,
       count(*) FILTER (WHERE client_session_id IS NULL) no_client_session,
       count(*) FILTER (WHERE gateway_conversation_id IS NULL) no_gateway_conversation
FROM ai_requests;
\echo == D22 stuck pending rows (ids)
SELECT id, provider, model, is_streaming, request_kind, client_kind, created_at::timestamp(0), updated_at::timestamp(0) FROM ai_requests WHERE status IN ('pending','streaming') ORDER BY created_at;
\echo == D22 status x finish_reason per day
SELECT created_at::date d, count(*) FILTER (WHERE status='completed') completed, count(*) FILTER (WHERE status='failed') failed, count(*) FILTER (WHERE status='rejected') rejected, count(*) FILTER (WHERE status='pending') pending, count(DISTINCT finish_reason) finish_reasons FROM ai_requests GROUP BY 1 ORDER BY 1;
\echo == D22 error_message shapes (failed/rejected) top 20
SELECT status, left(regexp_replace(error_message, '[0-9a-f]{8}-[0-9a-f-]{27}|[0-9]+', '#', 'g'), 110) shape, count(*) FROM ai_requests WHERE error_message IS NOT NULL GROUP BY 1,2 ORDER BY 3 DESC LIMIT 20;
\echo == D22 client attribution: client_kind x wire_protocol x attestation presence x session_source
SELECT r.client_kind, r.wire_protocol, r.client_attestation IS NOT NULL AS attested, s.session_source, s.client_type, count(*) FROM ai_requests r JOIN user_sessions s ON s.session_id = r.session_id GROUP BY 1,2,3,4,5 ORDER BY 6 DESC;
\echo == D22 client_attestation shapes (keys only)
SELECT (SELECT string_agg(k, ',') FROM jsonb_object_keys(client_attestation::jsonb) k) keys, count(*) FROM ai_requests WHERE client_attestation IS NOT NULL GROUP BY 1 ORDER BY 2 DESC LIMIT 5;
\echo == D22 latency: p50/p95/max by model (completed turns)
SELECT model, count(*), percentile_cont(0.5) WITHIN GROUP (ORDER BY latency_ms)::int p50, percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms)::int p95, max(latency_ms) FROM ai_requests WHERE status='completed' AND request_kind='turn' GROUP BY 1 ORDER BY 2 DESC LIMIT 8;
\echo == D22 request_id vs id (gateway path uses the same value; internal path differs)
SELECT count(*) FILTER (WHERE id = request_id) same, count(*) FILTER (WHERE id <> request_id) different FROM ai_requests;
\echo == D22 probes: what are they (model, client, cost)
SELECT model, client_kind, count(*), round(sum(cost_microdollars)/1e6, 2) usd, round(avg(latency_ms)) avg_ms FROM ai_requests WHERE request_kind='probe' GROUP BY 1,2 ORDER BY 3 DESC LIMIT 8;
