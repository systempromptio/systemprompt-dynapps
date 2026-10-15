-- Governance decisions a run's tool calls produced, joined on the run's
-- gateway and host sessions or its conversation context.
SELECT id AS "id!",
       created_at AS "created_at!",
       tool_name AS "tool_name!",
       decision AS "decision!",
       policy AS "policy!",
       reason AS "reason!",
       actor_kind AS "actor_kind!",
       actor_id AS "actor_id!",
       user_id AS "user_id!: UserId"
FROM governance_decisions
WHERE session_id = ANY($1::text[])
   OR ($2::text IS NOT NULL AND context_id = $2)
ORDER BY created_at
LIMIT $3
