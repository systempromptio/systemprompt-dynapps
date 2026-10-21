SELECT EXISTS(
    SELECT 1 FROM agent_farm_eval_runs WHERE id = $1 AND status = 'cancelled'
) AS "cancelled!"
