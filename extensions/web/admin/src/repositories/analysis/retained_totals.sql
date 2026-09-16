SELECT count(*) FILTER (WHERE fact_kind='invocation') AS invocations,
 count(*) FILTER (WHERE fact_kind='invocation' AND value->'skill'->>'plugin_id' IS NOT NULL) AS attributed,
 count(*) FILTER (WHERE fact_kind='invocation' AND resource_id IS NOT NULL) AS revision_verified,
 count(DISTINCT (consumer_id,value->'consumer'->>'session_id')) FILTER (WHERE fact_kind='invocation' AND value->'skill'->>'plugin_id' IS NOT NULL AND consumer_id IS NOT NULL) AS conversations,
 count(*) FILTER (WHERE fact_kind='request') AS requests,
 count(*) FILTER (WHERE fact_kind='request' AND NOT succeeded) AS failed,
 coalesce(sum(coalesce(input_tokens,0)+coalesce(output_tokens,0)) FILTER (WHERE fact_kind='request'),0)::bigint AS tokens,
 coalesce(sum(amount) FILTER (WHERE fact_kind='request' AND currency='USD'),0)::bigint AS cost
FROM analytics_snapshot_contributions
WHERE owner_id=$1 AND scope='' AND occurred_at>=$2 AND occurred_at<$3
