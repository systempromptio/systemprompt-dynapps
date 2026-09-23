-- Average and p99 size of the large text/jsonb columns.
SELECT 'ai_request_payloads.request_body' AS col, count(*) AS rows,
       pg_size_pretty(sum(octet_length(request_body::text))::bigint) AS total,
       pg_size_pretty(avg(octet_length(request_body::text))::bigint) AS avg,
       pg_size_pretty(percentile_cont(0.99) WITHIN GROUP (ORDER BY octet_length(request_body::text))::bigint) AS p99
FROM ai_request_payloads
UNION ALL
SELECT 'ai_request_payloads.offered_tools', count(*), pg_size_pretty(sum(octet_length(offered_tools::text))::bigint),
       pg_size_pretty(avg(octet_length(offered_tools::text))::bigint),
       pg_size_pretty(percentile_cont(0.99) WITHIN GROUP (ORDER BY octet_length(offered_tools::text))::bigint)
FROM ai_request_payloads
UNION ALL
SELECT 'ai_request_messages.content', count(*), pg_size_pretty(sum(octet_length(content))::bigint),
       pg_size_pretty(avg(octet_length(content))::bigint),
       pg_size_pretty(percentile_cont(0.99) WITHIN GROUP (ORDER BY octet_length(content))::bigint)
FROM ai_request_messages
UNION ALL
SELECT 'logs.metadata', count(*), pg_size_pretty(sum(octet_length(metadata::text))::bigint),
       pg_size_pretty(avg(octet_length(metadata::text))::bigint),
       pg_size_pretty(percentile_cont(0.99) WITHIN GROUP (ORDER BY octet_length(metadata::text))::bigint)
FROM logs
UNION ALL
SELECT 'governance_decisions.evaluated_rules', count(*), pg_size_pretty(sum(octet_length(evaluated_rules::text))::bigint),
       pg_size_pretty(avg(octet_length(evaluated_rules::text))::bigint),
       pg_size_pretty(percentile_cont(0.99) WITHIN GROUP (ORDER BY octet_length(evaluated_rules::text))::bigint)
FROM governance_decisions
UNION ALL
SELECT 'event_outbox.payload', count(*), pg_size_pretty(sum(octet_length(payload::text))::bigint),
       pg_size_pretty(avg(octet_length(payload::text))::bigint),
       pg_size_pretty(percentile_cont(0.99) WITHIN GROUP (ORDER BY octet_length(payload::text))::bigint)
FROM event_outbox;
