SELECT m.id AS resource_id,m.resource_key,m.kind,
 count(*) FILTER (WHERE c.fact_kind='invocation') AS invocations,
 count(DISTINCT (c.consumer_id,c.value->'consumer'->>'session_id')) FILTER (WHERE c.fact_kind='invocation' AND c.consumer_id IS NOT NULL) AS conversations,
 count(*) FILTER (WHERE c.fact_kind='request') AS requests,
 count(*) FILTER (WHERE c.fact_kind='request' AND NOT c.succeeded) AS failed,
 coalesce(sum(coalesce(c.input_tokens,0)+coalesce(c.output_tokens,0)) FILTER (WHERE c.fact_kind='request'),0)::bigint AS tokens,
 coalesce(sum(c.amount) FILTER (WHERE c.fact_kind='request' AND c.currency='USD'),0)::bigint AS cost,
 count(*) OVER() AS total_rows
FROM analytics_snapshot_contributions c
JOIN managed_resources m ON m.owner_id=c.owner_id AND m.id=c.scope
WHERE c.owner_id=$1 AND c.scope<>'' AND c.occurred_at>=$2 AND c.occurred_at<$3
GROUP BY m.id,m.resource_key,m.kind
ORDER BY invocations DESC,m.resource_key,m.id
LIMIT $4 OFFSET $5
