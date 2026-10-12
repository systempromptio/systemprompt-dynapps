INSERT INTO agent_farm_run_events (run_id, seq, kind, tool_name, tool_use_id, payload)
SELECT $1, e.seq, e.kind, e.tool_name, e.tool_use_id, e.payload
FROM UNNEST($2::bigint[], $3::text[], $4::text[], $5::text[], $6::jsonb[])
    AS e(seq, kind, tool_name, tool_use_id, payload)
ON CONFLICT (run_id, seq) DO NOTHING
