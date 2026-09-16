WITH inv AS (
 SELECT source,fact_id,consumer_id,session_id,host,occurred_at
 FROM analytics_normalized_facts
 WHERE owner_id=$1 AND NOT deleted AND fact_kind='invocation' AND fact->'value'->'skill'->>'plugin_id'=$2 AND fact->'value'->'skill'->>'skill'=$3
  AND consumer_id IS NOT NULL AND session_id IS NOT NULL AND occurred_at>=$4 AND occurred_at<$5
), conv AS (
 SELECT consumer_id,session_id,min(host) AS host,min(occurred_at) AS first_at,max(occurred_at) AS last_at,count(*) AS invocations
 FROM inv GROUP BY consumer_id,session_id
), req AS (
 SELECT DISTINCT i.consumer_id,i.session_id,r.source,r.fact_id,r.succeeded,r.input_tokens,r.output_tokens,r.amount_micros,r.currency
 FROM inv i
 JOIN analytics_normalized_facts a ON a.owner_id=$1 AND NOT a.deleted AND a.fact_kind='resource_association'
  AND a.invocation_source=i.source AND a.invocation_id=i.fact_id
 JOIN analytics_normalized_facts r ON r.owner_id=$1 AND NOT r.deleted AND r.fact_kind='request'
  AND r.source=a.request_source AND r.fact_id=a.request_id AND r.occurred_at>=$4 AND r.occurred_at<$5
), spend AS (
 SELECT consumer_id,session_id,count(*) AS requests,count(*) FILTER (WHERE NOT succeeded) AS failed,
  coalesce(sum(coalesce(input_tokens,0)+coalesce(output_tokens,0)),0)::bigint AS tokens,
  coalesce(sum(amount_micros) FILTER (WHERE currency='USD'),0)::bigint AS cost
 FROM req GROUP BY consumer_id,session_id
), assessed AS (
 SELECT DISTINCT ON (i.consumer_id,i.session_id) i.consumer_id,i.session_id,s.assessment_status,s.score_millionths
 FROM inv i
 JOIN analytics_normalized_facts s ON s.owner_id=$1 AND NOT s.deleted AND s.fact_kind='assessment'
  AND s.invocation_source=i.source AND s.invocation_id=i.fact_id
 ORDER BY i.consumer_id,i.session_id,s.occurred_at DESC,s.source,s.fact_id
)
SELECT c.session_id,c.consumer_id AS user_id,c.host,c.first_at,c.last_at,c.invocations,
 coalesce(s.requests,0)::bigint AS requests,coalesce(s.failed,0)::bigint AS failed,
 coalesce(s.tokens,0)::bigint AS tokens,coalesce(s.cost,0)::bigint AS cost,
 a.assessment_status,a.score_millionths,count(*) OVER() AS total_rows
FROM conv c
LEFT JOIN spend s ON s.consumer_id=c.consumer_id AND s.session_id=c.session_id
LEFT JOIN assessed a ON a.consumer_id=c.consumer_id AND a.session_id=c.session_id
ORDER BY c.last_at DESC,c.session_id,c.consumer_id
LIMIT $6 OFFSET $7
