WITH inv AS (
 SELECT f.source,f.fact_id,f.consumer_id,f.session_id,f.succeeded,
  f.fact->'value'->'skill'->>'marketplace_id' AS marketplace_id,
  coalesce(f.fact->'value'->'skill'->>'marketplace_hash',
           marketplace_version_at(f.fact->'value'->'skill'->>'marketplace_id',f.occurred_at)) AS marketplace_hash
 FROM analytics_normalized_facts f
 WHERE f.owner_id=$1 AND NOT f.deleted AND f.fact_kind='invocation' AND f.occurred_at>=$2 AND f.occurred_at<$3
  AND f.fact->'value'->'skill'->>'marketplace_id'=$4
), keyed AS (
 SELECT marketplace_id,marketplace_hash,count(*) AS invocations,
  count(*) FILTER (WHERE succeeded IS FALSE) AS failed_invocations,
  count(DISTINCT consumer_id) FILTER (WHERE consumer_id IS NOT NULL) AS users,
  count(DISTINCT (consumer_id,session_id)) FILTER (WHERE consumer_id IS NOT NULL AND session_id IS NOT NULL) AS conversations
 FROM inv GROUP BY 1,2
), req AS (
 SELECT DISTINCT i.marketplace_id,i.marketplace_hash,r.source,r.fact_id,r.succeeded,r.input_tokens,r.output_tokens,r.amount_micros,r.currency,r.latency_micros
 FROM inv i
 JOIN analytics_normalized_facts a ON a.owner_id=$1 AND NOT a.deleted AND a.fact_kind='resource_association'
  AND a.invocation_source=i.source AND a.invocation_id=i.fact_id
 JOIN analytics_normalized_facts r ON r.owner_id=$1 AND NOT r.deleted AND r.fact_kind='request'
  AND r.source=a.request_source AND r.fact_id=a.request_id AND r.occurred_at>=$2 AND r.occurred_at<$3
), spend AS (
 SELECT marketplace_id,marketplace_hash,count(*) AS requests,count(*) FILTER (WHERE NOT succeeded) AS failed,
  coalesce(sum(coalesce(input_tokens,0)+coalesce(output_tokens,0)),0)::bigint AS tokens,
  coalesce(sum(amount_micros) FILTER (WHERE currency='USD'),0)::bigint AS cost,
  percentile_cont(0.5) WITHIN GROUP (ORDER BY latency_micros) FILTER (WHERE latency_micros IS NOT NULL) AS p50_micros,
  percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_micros) FILTER (WHERE latency_micros IS NOT NULL) AS p95_micros,
  count(*) FILTER (WHERE latency_micros IS NOT NULL) AS latency_measured
 FROM req GROUP BY 1,2
)
SELECT v.marketplace_id AS "marketplace_id: MarketplaceId",v.content_hash,v.source,v.source_hash,v.origin,
 v.manifest AS "manifest: Json<MarketplaceManifest>",
 v.first_seen_at,v.last_seen_at,v.effective_until,v.plugin_count,v.skill_count,
 coalesce(k.invocations,0)::bigint AS "invocations!",coalesce(k.failed_invocations,0)::bigint AS "failed_invocations!",
 coalesce(k.users,0)::bigint AS "users!",coalesce(k.conversations,0)::bigint AS "conversations!",
 coalesce(s.requests,0)::bigint AS "requests!",coalesce(s.failed,0)::bigint AS "failed!",
 coalesce(s.tokens,0)::bigint AS "tokens!",coalesce(s.cost,0)::bigint AS "cost!",
 s.p50_micros::bigint AS p50_micros,s.p95_micros::bigint AS p95_micros,
 coalesce(s.latency_measured,0)::bigint AS "latency_measured!"
FROM marketplace_versions v
LEFT JOIN keyed k ON k.marketplace_id=v.marketplace_id AND k.marketplace_hash=v.content_hash
LEFT JOIN spend s ON s.marketplace_id=v.marketplace_id AND s.marketplace_hash=v.content_hash
WHERE v.marketplace_id=$4
ORDER BY (v.effective_until IS NULL) DESC,v.first_seen_at DESC
