WITH cur AS (
 SELECT marketplace_id,content_hash,source,source_hash,first_seen_at,plugin_count,skill_count,manifest->>'name' AS name
 FROM marketplace_versions WHERE effective_until IS NULL
), hist AS (
 SELECT marketplace_id,count(*) AS versions,min(first_seen_at) AS first_seen_at,max(first_seen_at) AS last_changed_at
 FROM marketplace_versions GROUP BY 1
), inv AS (
 SELECT f.source,f.fact_id,f.consumer_id,f.succeeded,f.fact->'value'->'skill'->>'marketplace_id' AS marketplace_id
 FROM analytics_normalized_facts f
 WHERE f.owner_id=$1 AND NOT f.deleted AND f.fact_kind='invocation' AND f.occurred_at>=$2 AND f.occurred_at<$3
  AND f.fact->'value'->'skill'->>'marketplace_id' IS NOT NULL
), keyed AS (
 SELECT marketplace_id,count(*) AS invocations,count(*) FILTER (WHERE succeeded IS FALSE) AS failed_invocations,
  count(DISTINCT consumer_id) FILTER (WHERE consumer_id IS NOT NULL) AS users
 FROM inv GROUP BY 1
), req AS (
 SELECT DISTINCT i.marketplace_id,r.source,r.fact_id,r.succeeded,r.amount_micros,r.currency
 FROM inv i
 JOIN analytics_normalized_facts a ON a.owner_id=$1 AND NOT a.deleted AND a.fact_kind='resource_association'
  AND a.invocation_source=i.source AND a.invocation_id=i.fact_id
 JOIN analytics_normalized_facts r ON r.owner_id=$1 AND NOT r.deleted AND r.fact_kind='request'
  AND r.source=a.request_source AND r.fact_id=a.request_id AND r.occurred_at>=$2 AND r.occurred_at<$3
), spend AS (
 SELECT marketplace_id,count(*) AS requests,count(*) FILTER (WHERE NOT succeeded) AS failed,
  coalesce(sum(amount_micros) FILTER (WHERE currency='USD'),0)::bigint AS cost
 FROM req GROUP BY 1
)
SELECT h.marketplace_id AS "marketplace_id: MarketplaceId",c.name,c.content_hash,c.source,c.source_hash,c.plugin_count,c.skill_count,
 h.versions::bigint AS "versions!",h.first_seen_at AS "first_seen_at!",h.last_changed_at AS "last_changed_at!",
 coalesce(k.invocations,0)::bigint AS "invocations!",coalesce(k.failed_invocations,0)::bigint AS "failed_invocations!",
 coalesce(k.users,0)::bigint AS "users!",coalesce(s.requests,0)::bigint AS "requests!",coalesce(s.failed,0)::bigint AS "failed!",
 coalesce(s.cost,0)::bigint AS "cost!"
FROM hist h
LEFT JOIN cur c ON c.marketplace_id=h.marketplace_id
LEFT JOIN keyed k ON k.marketplace_id=h.marketplace_id
LEFT JOIN spend s ON s.marketplace_id=h.marketplace_id
ORDER BY (c.content_hash IS NULL),k.invocations DESC NULLS LAST,h.marketplace_id
