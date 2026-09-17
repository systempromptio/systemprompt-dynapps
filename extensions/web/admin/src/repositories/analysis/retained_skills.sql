WITH inv AS (
 SELECT source,fact_id,consumer_id,session_id,resource_id,
  fact->'value'->'skill'->>'plugin_id' AS plugin_id,fact->'value'->'skill'->>'skill' AS skill,
  fact->'value'->'skill'->>'marketplace_id' AS marketplace_id,fact->'value'->'skill'->>'source' AS src,
  fact->'value'->'skill'->>'source_hash' AS source_hash
 FROM analytics_normalized_facts
 WHERE owner_id=$1 AND NOT deleted AND fact_kind='invocation' AND occurred_at>=$2 AND occurred_at<$3
  AND fact->'value'->'skill'->>'plugin_id' IS NOT NULL
), keyed AS (
 SELECT plugin_id,skill,marketplace_id,src,source_hash,
  count(*) AS invocations,count(*) FILTER (WHERE resource_id IS NOT NULL) AS revision_verified,
  count(DISTINCT (consumer_id,session_id)) FILTER (WHERE consumer_id IS NOT NULL AND session_id IS NOT NULL) AS conversations,
  max(resource_id) AS resource_id
 FROM inv GROUP BY plugin_id,skill,marketplace_id,src,source_hash
), req AS (
 SELECT DISTINCT i.plugin_id,i.skill,i.marketplace_id,i.src,i.source_hash,
  r.source,r.fact_id,r.succeeded,r.input_tokens,r.output_tokens,r.amount_micros,r.currency
 FROM inv i
 JOIN analytics_normalized_facts a ON a.owner_id=$1 AND NOT a.deleted AND a.fact_kind='resource_association'
  AND a.invocation_source=i.source AND a.invocation_id=i.fact_id
 JOIN analytics_normalized_facts r ON r.owner_id=$1 AND NOT r.deleted AND r.fact_kind='request'
  AND r.source=a.request_source AND r.fact_id=a.request_id AND r.occurred_at>=$2 AND r.occurred_at<$3
), spend AS (
 SELECT plugin_id,skill,marketplace_id,src,source_hash,
  count(*) AS requests,count(*) FILTER (WHERE NOT succeeded) AS failed,
  coalesce(sum(coalesce(input_tokens,0)+coalesce(output_tokens,0)),0)::bigint AS tokens,
  coalesce(sum(amount_micros) FILTER (WHERE currency='USD'),0)::bigint AS cost
 FROM req GROUP BY plugin_id,skill,marketplace_id,src,source_hash
), art AS (
 -- Artifacts attributed to the skill: each artifact fact names the skill it
 -- ran under the same way an invocation does.
 SELECT fact->'value'->'skill'->>'plugin_id' AS plugin_id,fact->'value'->'skill'->>'skill' AS skill,
  fact->'value'->'skill'->>'marketplace_id' AS marketplace_id,fact->'value'->'skill'->>'source' AS src,
  fact->'value'->'skill'->>'source_hash' AS source_hash,
  count(*) AS artifacts,
  count(*) FILTER (WHERE (fact->'value'->>'is_structured')::boolean) AS structured_artifacts
 FROM analytics_normalized_facts
 WHERE owner_id=$1 AND NOT deleted AND fact_kind='artifact' AND occurred_at>=$2 AND occurred_at<$3
  AND fact->'value'->'skill'->>'plugin_id' IS NOT NULL
 GROUP BY 1,2,3,4,5
)
SELECT k.plugin_id,k.skill,k.marketplace_id,k.src AS source,k.source_hash,
 k.resource_id,m.resource_key,m.kind,
 k.invocations::bigint AS invocations,k.revision_verified::bigint AS revision_verified,k.conversations::bigint AS conversations,
 coalesce(s.requests,0)::bigint AS requests,coalesce(s.failed,0)::bigint AS failed,
 coalesce(s.tokens,0)::bigint AS tokens,coalesce(s.cost,0)::bigint AS cost,
 coalesce(ar.artifacts,0)::bigint AS artifacts,coalesce(ar.structured_artifacts,0)::bigint AS structured_artifacts,
 count(*) OVER() AS total_rows
FROM keyed k
LEFT JOIN managed_resources m ON m.owner_id=$1 AND m.id=k.resource_id
LEFT JOIN spend s ON s.plugin_id=k.plugin_id AND s.skill=k.skill
 AND s.marketplace_id IS NOT DISTINCT FROM k.marketplace_id AND s.src IS NOT DISTINCT FROM k.src
 AND s.source_hash IS NOT DISTINCT FROM k.source_hash
LEFT JOIN art ar ON ar.plugin_id=k.plugin_id AND ar.skill=k.skill
 AND ar.marketplace_id IS NOT DISTINCT FROM k.marketplace_id AND ar.src IS NOT DISTINCT FROM k.src
 AND ar.source_hash IS NOT DISTINCT FROM k.source_hash
ORDER BY k.invocations DESC,k.plugin_id,k.skill,k.source_hash
LIMIT $4 OFFSET $5
