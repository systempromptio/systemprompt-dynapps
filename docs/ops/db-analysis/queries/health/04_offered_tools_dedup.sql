-- How much of offered_tools is duplicate: distinct digests vs rows.
SELECT count(*) AS rows_with_tools,
       count(DISTINCT md5(offered_tools::text)) AS distinct_tool_lists,
       pg_size_pretty(sum(octet_length(offered_tools::text))::bigint) AS stored,
       pg_size_pretty((SELECT sum(octet_length(t))::bigint FROM (SELECT DISTINCT offered_tools::text AS t FROM ai_request_payloads WHERE offered_tools IS NOT NULL) d)) AS if_deduplicated
FROM ai_request_payloads WHERE offered_tools IS NOT NULL;
