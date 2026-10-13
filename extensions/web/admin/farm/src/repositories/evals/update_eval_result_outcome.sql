UPDATE agent_farm_eval_results
SET status = $2, assertions = $3, judge_score = $4, judge_rationale = $5, updated_at = NOW()
WHERE id = $1
