WITH inv AS (
 SELECT f.source,f.fact_id,f.consumer_id,
  f.fact->'value'->'skill'->>'plugin_id' AS plugin_id,f.fact->'value'->'skill'->>'skill' AS skill
 FROM analytics_normalized_facts f
 WHERE f.owner_id=$1 AND NOT f.deleted AND f.fact_kind='invocation' AND f.occurred_at>=$2 AND f.occurred_at<$3
  AND f.fact->'value'->'skill'->>'plugin_id' IS NOT NULL
), keyed AS (
 SELECT plugin_id,skill,count(DISTINCT consumer_id) FILTER (WHERE consumer_id IS NOT NULL) AS users
 FROM inv GROUP BY 1,2
), req AS (
 SELECT DISTINCT i.plugin_id,i.skill,r.source,r.fact_id,r.latency_micros
 FROM inv i
 JOIN analytics_normalized_facts a ON a.owner_id=$1 AND NOT a.deleted AND a.fact_kind='resource_association'
  AND a.invocation_source=i.source AND a.invocation_id=i.fact_id
 JOIN analytics_normalized_facts r ON r.owner_id=$1 AND NOT r.deleted AND r.fact_kind='request'
  AND r.source=a.request_source AND r.fact_id=a.request_id AND r.occurred_at>=$2 AND r.occurred_at<$3
), lat AS (
 SELECT plugin_id,skill,
  percentile_cont(0.5) WITHIN GROUP (ORDER BY latency_micros) FILTER (WHERE latency_micros IS NOT NULL) AS p50_micros,
  percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_micros) FILTER (WHERE latency_micros IS NOT NULL) AS p95_micros,
  count(*) FILTER (WHERE latency_micros IS NOT NULL) AS latency_measured
 FROM req GROUP BY 1,2
)
SELECT k.plugin_id AS "plugin_id!: PluginId",k.skill AS "skill!",k.users::bigint AS "users!",
 l.p50_micros::bigint AS p50_micros,l.p95_micros::bigint AS p95_micros,coalesce(l.latency_measured,0)::bigint AS "latency_measured!"
FROM keyed k LEFT JOIN lat l ON l.plugin_id=k.plugin_id AND l.skill=k.skill
