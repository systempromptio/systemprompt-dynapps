-- C11a: tokens_used vs billable formula (input+output+cache_read+cache_creation; reasoning excluded — CanonicalUsage::billable_total)
SELECT status, count(*) n,
  count(*) FILTER (WHERE tokens_used IS DISTINCT FROM COALESCE(input_tokens,0)+COALESCE(output_tokens,0)+COALESCE(cache_read_tokens,0)+COALESCE(cache_creation_tokens,0)) formula_mismatch,
  count(*) FILTER (WHERE tokens_used = COALESCE(input_tokens,0)+COALESCE(output_tokens,0)) io_only,
  count(*) FILTER (WHERE tokens_used IS NULL) tokens_null, count(*) FILTER (WHERE tokens_used=0) tokens_zero,
  count(*) FILTER (WHERE reasoning_tokens>0) has_reasoning, count(*) FILTER (WHERE reasoning_tokens > output_tokens) reasoning_gt_output
FROM ai_requests GROUP BY 1 ORDER BY 2 DESC;
SELECT id, provider, model, tokens_used, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, reasoning_tokens FROM ai_requests
WHERE tokens_used IS DISTINCT FROM COALESCE(input_tokens,0)+COALESCE(output_tokens,0)+COALESCE(cache_read_tokens,0)+COALESCE(cache_creation_tokens,0) AND status='completed' LIMIT 5;
-- C11b: uncosted completed requests by provider/model
SELECT provider, served_provider, model, request_kind, count(*) n, count(*) FILTER (WHERE cost_microdollars=0 AND tokens_used>0) uncosted, sum(tokens_used) tokens, round(sum(cost_microdollars)/1e6,2) usd,
       round(avg(cost_microdollars)::numeric/1e6,4) avg_usd
FROM ai_requests WHERE status='completed' GROUP BY 1,2,3,4 ORDER BY 5 DESC;
-- C11c: recompute cost from providers.yaml for the top models (input/output only + cache read at 10%/creation 125% anthropic default)
SELECT model, count(*) n, round(sum(cost_microdollars)/1e6,2) stored_usd,
  round(sum( input_tokens*p.i + output_tokens*p.o + COALESCE(cache_read_tokens,0)*p.cr + COALESCE(cache_creation_tokens,0)*p.cc )/1e6,2) recomputed_usd
FROM ai_requests r JOIN (VALUES ('claude-opus-4-8',5.0,25.0,0.5,6.25),('claude-sonnet-4-6',3.0,15.0,0.3,3.75),('claude-opus-4-7',5.0,25.0,0.5,6.25),('claude-haiku-4-5',1.0,5.0,0.1,1.25),('claude-sonnet-4-5',3.0,15.0,0.3,3.75),('claude-opus-4-6',5.0,25.0,0.5,6.25),('claude-opus-4-5',5.0,25.0,0.5,6.25),('claude-sonnet-5',2.0,10.0,0.2,2.5),('claude-opus-5',5.0,25.0,0.5,6.25)) p(m,i,o,cr,cc) ON p.m=r.model
WHERE status='completed' GROUP BY 1 ORDER BY 2 DESC;
SELECT count(*) FILTER (WHERE status='completed' AND cost_microdollars=0 AND tokens_used>0) uncosted_total, count(*) FILTER (WHERE status='completed') completed FROM ai_requests;
