-- C18: mcp_artifacts / artifact_parts / artifact_payloads
\echo == C18 artifacts overview
SELECT count(*) artifacts, count(DISTINCT mcp_execution_id) executions, count(*) FILTER (WHERE NOT EXISTS (SELECT 1 FROM mcp_tool_executions e WHERE e.mcp_execution_id = a.mcp_execution_id)) orphan_execution,
       count(*) FILTER (WHERE ai_tool_call_id IS NOT NULL) with_tool_call, count(*) FILTER (WHERE payload_sha256 IS NOT NULL) with_payload_sha,
       count(*) FILTER (WHERE payload_sha256 IS NOT NULL AND NOT EXISTS (SELECT 1 FROM artifact_payloads p WHERE p.sha256 = a.payload_sha256)) sha_without_payload,
       count(*) FILTER (WHERE is_error) errors FROM mcp_artifacts a;
SELECT source, last_seen_source, artifact_type, count(*) FROM mcp_artifacts GROUP BY 1,2,3 ORDER BY 4 DESC LIMIT 8;
\echo == C18 artifact_parts: parts without artifacts, artifacts without parts
SELECT (SELECT count(*) FROM artifact_parts) parts,
       (SELECT count(*) FROM artifact_parts p WHERE NOT EXISTS (SELECT 1 FROM mcp_artifacts a WHERE a.artifact_id = p.artifact_id)) parts_without_artifact,
       (SELECT count(*) FROM mcp_artifacts a WHERE NOT EXISTS (SELECT 1 FROM artifact_parts p WHERE p.artifact_id = a.artifact_id)) artifacts_without_parts;
\echo == C18 payload store: ref_count vs actual references, sizes
SELECT count(*) payloads, pg_size_pretty(sum(byte_len)::bigint) bytes, pg_size_pretty(max(byte_len)::bigint) largest,
       count(*) FILTER (WHERE ref_count <> (SELECT count(*) FROM mcp_artifacts a WHERE a.payload_sha256 = p.sha256)) refcount_wrong,
       count(*) FILTER (WHERE ref_count = 0) unreferenced FROM artifact_payloads p;
\echo == C18 tool_activity.artifact_kind (what the console counts as an artifact) vs raw artifacts
SELECT (SELECT count(*) FROM mcp_artifacts) raw, (SELECT count(*) FROM tool_activity WHERE artifact_kind IS NOT NULL) kind_not_null,
       (SELECT string_agg(coalesce(artifact_kind,'<null>')||'='||n, ', ') FROM (SELECT artifact_kind, count(*) n FROM tool_activity GROUP BY 1) k) kinds;
