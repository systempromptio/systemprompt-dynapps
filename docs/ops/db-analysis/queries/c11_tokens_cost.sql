-- C11: tokens_used identity and costing
\echo == C11 tokens_used identity: which formula holds
SELECT count(*) total,
  count(*) FILTER (WHERE tokens_used = coalesce(input_tokens,0)+coalesce(output_tokens,0)) AS in_plus_out,
  count(*) FILTER (WHERE tokens_used = coalesce(input_tokens,0)+coalesce(output_tokens,0)+coalesce(cache_read_tokens,0)+coalesce(cache_creation_tokens,0)) AS in_out_cache,
  count(*) FILTER (WHERE tokens_used = coalesce(input_tokens,0)+coalesce(output_tokens,0)+coalesce(cache_read_tokens,0)+coalesce(cache_creation_tokens,0)+coalesce(reasoning_tokens,0)) AS in_out_cache_reasoning,
  count(*) FILTER (WHERE reasoning_tokens > 0) AS has_reasoning, sum(reasoning_tokens) AS reasoning_total,
  count(*) FILTER (WHERE output_tokens >= reasoning_tokens AND reasoning_tokens > 0) AS reasoning_within_output
FROM ai_requests;
\echo == C11 uncosted completed requests with tokens, by provider/model
SELECT provider, model, count(*) FROM ai_requests WHERE status = 'completed' AND tokens_used > 0 AND cost_microdollars = 0 GROUP BY 1,2 ORDER BY 3 DESC;
\echo == C11 completed with zero tokens (settlement never landed usage)
SELECT provider, model, is_streaming, request_kind, count(*) FROM ai_requests WHERE status = 'completed' AND coalesce(tokens_used,0) = 0 GROUP BY 1,2,3,4 ORDER BY 5 DESC LIMIT 10;
\echo == C11 cost per provider/model: requests, tokens, cost, USD per 1M tokens
SELECT provider, model, count(*) n, sum(tokens_used) tokens, round(sum(cost_microdollars)/1e6,2) usd, round((sum(cost_microdollars)::numeric/1e6)/nullif(sum(tokens_used),0)*1e6,2) usd_per_mtok
FROM ai_requests WHERE status='completed' GROUP BY 1,2 ORDER BY 5 DESC LIMIT 15;
\echo == C11 accounting failures
SELECT count(*) FILTER (WHERE accounting_failed_at IS NOT NULL) failed, left(min(accounting_error),80) example FROM ai_requests;
\echo == C11 probe requests: are they costed / tokened
SELECT request_kind, count(*), sum(tokens_used) tokens, round(sum(cost_microdollars)/1e6,4) usd, count(*) FILTER (WHERE synthetic) synthetic FROM ai_requests GROUP BY 1;
