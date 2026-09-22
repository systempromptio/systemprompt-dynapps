-- B9: for every logical reference column (user_id, session_id, context_id, request_id, task_id, group_id, project_id, mcp_execution_id, ai_tool_call_id/tool_use_id), rows whose value has no parent.
-- Deleted-crate leftovers (eval_*, knowledge_*, requirements_*) and analytics_report_* projections excluded.
\echo == B9 orphan references (only columns with orphans > 0 or nulls on spine tables shown; full list in the query)
SELECT * FROM (
SELECT 'admin_usage_daily_rollups.group_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.group_id IS NULL) AS nulls, count(*) FILTER (WHERE x.group_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM groups p WHERE p.id::text = x.group_id::text)) AS orphans FROM admin_usage_daily_rollups x
UNION ALL
SELECT 'admin_usage_daily_rollups.project_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.project_id IS NULL) AS nulls, count(*) FILTER (WHERE x.project_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM projects p WHERE p.id::text = x.project_id::text)) AS orphans FROM admin_usage_daily_rollups x
UNION ALL
SELECT 'admin_usage_daily_rollups.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM admin_usage_daily_rollups x
UNION ALL
SELECT 'agent_tasks.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM agent_tasks x
UNION ALL
SELECT 'agent_tasks.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM agent_tasks x
UNION ALL
SELECT 'agent_tasks.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM agent_tasks x
UNION ALL
SELECT 'ai_gateway_thought_signatures.tool_use_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.tool_use_id IS NULL) AS nulls, count(*) FILTER (WHERE x.tool_use_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_request_tool_calls p WHERE p.ai_tool_call_id::text = x.tool_use_id::text)) AS orphans FROM ai_gateway_thought_signatures x
UNION ALL
SELECT 'ai_gateway_thought_signatures.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM ai_gateway_thought_signatures x
UNION ALL
SELECT 'ai_request_client_evidence.ai_request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.ai_request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.ai_request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.ai_request_id::text)) AS orphans FROM ai_request_client_evidence x
UNION ALL
SELECT 'ai_request_messages.request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.request_id::text)) AS orphans FROM ai_request_messages x
UNION ALL
SELECT 'ai_request_payloads.ai_request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.ai_request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.ai_request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.ai_request_id::text)) AS orphans FROM ai_request_payloads x
UNION ALL
SELECT 'ai_request_scopes.group_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.group_id IS NULL) AS nulls, count(*) FILTER (WHERE x.group_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM groups p WHERE p.id::text = x.group_id::text)) AS orphans FROM ai_request_scopes x
UNION ALL
SELECT 'ai_request_scopes.project_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.project_id IS NULL) AS nulls, count(*) FILTER (WHERE x.project_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM projects p WHERE p.id::text = x.project_id::text)) AS orphans FROM ai_request_scopes x
UNION ALL
SELECT 'ai_request_scopes.request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.request_id::text)) AS orphans FROM ai_request_scopes x
UNION ALL
SELECT 'ai_request_scopes.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM ai_request_scopes x
UNION ALL
SELECT 'ai_request_tool_calls.mcp_execution_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.mcp_execution_id IS NULL) AS nulls, count(*) FILTER (WHERE x.mcp_execution_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM mcp_tool_executions p WHERE p.mcp_execution_id::text = x.mcp_execution_id::text)) AS orphans FROM ai_request_tool_calls x
UNION ALL
SELECT 'ai_request_tool_calls.request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.request_id::text)) AS orphans FROM ai_request_tool_calls x
UNION ALL
SELECT 'ai_requests.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM ai_requests x
UNION ALL
SELECT 'ai_requests.mcp_execution_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.mcp_execution_id IS NULL) AS nulls, count(*) FILTER (WHERE x.mcp_execution_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM mcp_tool_executions p WHERE p.mcp_execution_id::text = x.mcp_execution_id::text)) AS orphans FROM ai_requests x
UNION ALL
SELECT 'ai_requests.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM ai_requests x
UNION ALL
SELECT 'ai_requests.task_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.task_id IS NULL) AS nulls, count(*) FILTER (WHERE x.task_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM agent_tasks p WHERE p.task_id::text = x.task_id::text)) AS orphans FROM ai_requests x
UNION ALL
SELECT 'ai_requests.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM ai_requests x
UNION ALL
SELECT 'ai_safety_findings.ai_request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.ai_request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.ai_request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.ai_request_id::text)) AS orphans FROM ai_safety_findings x
UNION ALL
SELECT 'analysis_reports.ai_request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.ai_request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.ai_request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.ai_request_id::text)) AS orphans FROM analysis_reports x
UNION ALL
SELECT 'analysis_skill_events.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM analysis_skill_events x
UNION ALL
SELECT 'analysis_skill_events.tool_use_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.tool_use_id IS NULL) AS nulls, count(*) FILTER (WHERE x.tool_use_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_request_tool_calls p WHERE p.ai_tool_call_id::text = x.tool_use_id::text)) AS orphans FROM analysis_skill_events x
UNION ALL
SELECT 'analysis_skill_events.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM analysis_skill_events x
UNION ALL
SELECT 'analysis_skill_version_events.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM analysis_skill_version_events x
UNION ALL
SELECT 'analysis_skill_version_events.tool_use_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.tool_use_id IS NULL) AS nulls, count(*) FILTER (WHERE x.tool_use_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_request_tool_calls p WHERE p.ai_tool_call_id::text = x.tool_use_id::text)) AS orphans FROM analysis_skill_version_events x
UNION ALL
SELECT 'analysis_skill_version_events.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM analysis_skill_version_events x
UNION ALL
SELECT 'analytics_events.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM analytics_events x
UNION ALL
SELECT 'analytics_events.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM analytics_events x
UNION ALL
SELECT 'analytics_events.task_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.task_id IS NULL) AS nulls, count(*) FILTER (WHERE x.task_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM agent_tasks p WHERE p.task_id::text = x.task_id::text)) AS orphans FROM analytics_events x
UNION ALL
SELECT 'analytics_events.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM analytics_events x
UNION ALL
SELECT 'analytics_normalized_facts.request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.request_id::text)) AS orphans FROM analytics_normalized_facts x
UNION ALL
SELECT 'analytics_normalized_facts.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM analytics_normalized_facts x
UNION ALL
SELECT 'analytics_snapshot_contributions.request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.request_id::text)) AS orphans FROM analytics_snapshot_contributions x
UNION ALL
SELECT 'analytics_snapshot_dimensions.request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.request_id::text)) AS orphans FROM analytics_snapshot_dimensions x
UNION ALL
SELECT 'approval_requests.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM approval_requests x
UNION ALL
SELECT 'artifact_parts.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM artifact_parts x
UNION ALL
SELECT 'bridge_exchange_codes.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM bridge_exchange_codes x
UNION ALL
SELECT 'bridge_sessions.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM bridge_sessions x
UNION ALL
SELECT 'bridge_sessions.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM bridge_sessions x
UNION ALL
SELECT 'bridge_user_host_model_prefs.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM bridge_user_host_model_prefs x
UNION ALL
SELECT 'bridge_user_host_prefs.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM bridge_user_host_prefs x
UNION ALL
SELECT 'context_agents.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM context_agents x
UNION ALL
SELECT 'context_notifications.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM context_notifications x
UNION ALL
SELECT 'conversation_analyses.ai_request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.ai_request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.ai_request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.ai_request_id::text)) AS orphans FROM conversation_analyses x
UNION ALL
SELECT 'conversation_analyses.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM conversation_analyses x
UNION ALL
SELECT 'conversation_analyses.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM conversation_analyses x
UNION ALL
SELECT 'conversation_analysis_rows.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM conversation_analysis_rows x
UNION ALL
SELECT 'conversation_analysis_rows.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM conversation_analysis_rows x
UNION ALL
SELECT 'conversation_analysis_rows.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM conversation_analysis_rows x
UNION ALL
SELECT 'conversation_facts.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM conversation_facts x
UNION ALL
SELECT 'conversation_facts.group_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.group_id IS NULL) AS nulls, count(*) FILTER (WHERE x.group_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM groups p WHERE p.id::text = x.group_id::text)) AS orphans FROM conversation_facts x
UNION ALL
SELECT 'conversation_facts.project_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.project_id IS NULL) AS nulls, count(*) FILTER (WHERE x.project_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM projects p WHERE p.id::text = x.project_id::text)) AS orphans FROM conversation_facts x
UNION ALL
SELECT 'conversation_facts.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM conversation_facts x
UNION ALL
SELECT 'conversation_facts.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM conversation_facts x
UNION ALL
SELECT 'conversation_requests.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM conversation_requests x
UNION ALL
SELECT 'conversation_requests.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM conversation_requests x
UNION ALL
SELECT 'conversation_requests.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM conversation_requests x
UNION ALL
SELECT 'conversation_rollups.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM conversation_rollups x
UNION ALL
SELECT 'conversation_rollups.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM conversation_rollups x
UNION ALL
SELECT 'conversation_rollups.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM conversation_rollups x
UNION ALL
SELECT 'conversation_skill_versions.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM conversation_skill_versions x
UNION ALL
SELECT 'conversation_skill_versions.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM conversation_skill_versions x
UNION ALL
SELECT 'daily_summaries.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM daily_summaries x
UNION ALL
SELECT 'dev_login_codes.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM dev_login_codes x
UNION ALL
SELECT 'device_app_links.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM device_app_links x
UNION ALL
SELECT 'engagement_events.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM engagement_events x
UNION ALL
SELECT 'engagement_events.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM engagement_events x
UNION ALL
SELECT 'event_outbox.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM event_outbox x
UNION ALL
SELECT 'federated_identities.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM federated_identities x
UNION ALL
SELECT 'feedback_ingestion_outbox.request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.request_id::text)) AS orphans FROM feedback_ingestion_outbox x
UNION ALL
SELECT 'feedback_ingestion_outbox.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM feedback_ingestion_outbox x
UNION ALL
SELECT 'files.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM files x
UNION ALL
SELECT 'files.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM files x
UNION ALL
SELECT 'files.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM files x
UNION ALL
SELECT 'funnel_progress.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM funnel_progress x
UNION ALL
SELECT 'governance_decisions.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM governance_decisions x
UNION ALL
SELECT 'governance_decisions.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM governance_decisions x
UNION ALL
SELECT 'governance_decisions.task_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.task_id IS NULL) AS nulls, count(*) FILTER (WHERE x.task_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM agent_tasks p WHERE p.task_id::text = x.task_id::text)) AS orphans FROM governance_decisions x
UNION ALL
SELECT 'governance_decisions.tool_use_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.tool_use_id IS NULL) AS nulls, count(*) FILTER (WHERE x.tool_use_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_request_tool_calls p WHERE p.ai_tool_call_id::text = x.tool_use_id::text)) AS orphans FROM governance_decisions x
UNION ALL
SELECT 'governance_decisions.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM governance_decisions x
UNION ALL
SELECT 'group_ad_mappings.group_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.group_id IS NULL) AS nulls, count(*) FILTER (WHERE x.group_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM groups p WHERE p.id::text = x.group_id::text)) AS orphans FROM group_ad_mappings x
UNION ALL
SELECT 'group_members.group_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.group_id IS NULL) AS nulls, count(*) FILTER (WHERE x.group_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM groups p WHERE p.id::text = x.group_id::text)) AS orphans FROM group_members x
UNION ALL
SELECT 'group_members.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM group_members x
UNION ALL
SELECT 'ingestion_repairs.request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.request_id::text)) AS orphans FROM ingestion_repairs x
UNION ALL
SELECT 'ingestion_session_owners.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM ingestion_session_owners x
UNION ALL
SELECT 'ingestion_session_owners.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM ingestion_session_owners x
UNION ALL
SELECT 'link_clicks.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM link_clicks x
UNION ALL
SELECT 'link_clicks.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM link_clicks x
UNION ALL
SELECT 'link_clicks.task_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.task_id IS NULL) AS nulls, count(*) FILTER (WHERE x.task_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM agent_tasks p WHERE p.task_id::text = x.task_id::text)) AS orphans FROM link_clicks x
UNION ALL
SELECT 'link_clicks.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM link_clicks x
UNION ALL
SELECT 'logs.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM logs x
UNION ALL
SELECT 'logs.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM logs x
UNION ALL
SELECT 'logs.task_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.task_id IS NULL) AS nulls, count(*) FILTER (WHERE x.task_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM agent_tasks p WHERE p.task_id::text = x.task_id::text)) AS orphans FROM logs x
UNION ALL
SELECT 'logs.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM logs x
UNION ALL
SELECT 'mcp_artifacts.ai_tool_call_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.ai_tool_call_id IS NULL) AS nulls, count(*) FILTER (WHERE x.ai_tool_call_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_request_tool_calls p WHERE p.ai_tool_call_id::text = x.ai_tool_call_id::text)) AS orphans FROM mcp_artifacts x
UNION ALL
SELECT 'mcp_artifacts.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM mcp_artifacts x
UNION ALL
SELECT 'mcp_artifacts.mcp_execution_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.mcp_execution_id IS NULL) AS nulls, count(*) FILTER (WHERE x.mcp_execution_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM mcp_tool_executions p WHERE p.mcp_execution_id::text = x.mcp_execution_id::text)) AS orphans FROM mcp_artifacts x
UNION ALL
SELECT 'mcp_artifacts.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM mcp_artifacts x
UNION ALL
SELECT 'mcp_artifacts.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM mcp_artifacts x
UNION ALL
SELECT 'mcp_connector_accounts.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM mcp_connector_accounts x
UNION ALL
SELECT 'mcp_connector_credentials.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM mcp_connector_credentials x
UNION ALL
SELECT 'mcp_connector_oauth_states.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM mcp_connector_oauth_states x
UNION ALL
SELECT 'mcp_external_sessions.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM mcp_external_sessions x
UNION ALL
SELECT 'mcp_external_sessions.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM mcp_external_sessions x
UNION ALL
SELECT 'mcp_proxy_identities.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM mcp_proxy_identities x
UNION ALL
SELECT 'mcp_proxy_identities.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM mcp_proxy_identities x
UNION ALL
SELECT 'mcp_sessions.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM mcp_sessions x
UNION ALL
SELECT 'mcp_sessions.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM mcp_sessions x
UNION ALL
SELECT 'mcp_tool_executions.ai_tool_call_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.ai_tool_call_id IS NULL) AS nulls, count(*) FILTER (WHERE x.ai_tool_call_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_request_tool_calls p WHERE p.ai_tool_call_id::text = x.ai_tool_call_id::text)) AS orphans FROM mcp_tool_executions x
UNION ALL
SELECT 'mcp_tool_executions.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM mcp_tool_executions x
UNION ALL
SELECT 'mcp_tool_executions.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM mcp_tool_executions x
UNION ALL
SELECT 'mcp_tool_executions.task_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.task_id IS NULL) AS nulls, count(*) FILTER (WHERE x.task_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM agent_tasks p WHERE p.task_id::text = x.task_id::text)) AS orphans FROM mcp_tool_executions x
UNION ALL
SELECT 'mcp_tool_executions.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM mcp_tool_executions x
UNION ALL
SELECT 'message_parts.task_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.task_id IS NULL) AS nulls, count(*) FILTER (WHERE x.task_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM agent_tasks p WHERE p.task_id::text = x.task_id::text)) AS orphans FROM message_parts x
UNION ALL
SELECT 'oauth_auth_codes.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM oauth_auth_codes x
UNION ALL
SELECT 'oauth_jti_revocations.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM oauth_jti_revocations x
UNION ALL
SELECT 'oauth_refresh_tokens.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM oauth_refresh_tokens x
UNION ALL
SELECT 'plugin_env_vars.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM plugin_env_vars x
UNION ALL
SELECT 'plugin_session_summaries.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM plugin_session_summaries x
UNION ALL
SELECT 'plugin_session_summaries.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM plugin_session_summaries x
UNION ALL
SELECT 'plugin_usage_daily.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM plugin_usage_daily x
UNION ALL
SELECT 'plugin_usage_events.mcp_execution_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.mcp_execution_id IS NULL) AS nulls, count(*) FILTER (WHERE x.mcp_execution_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM mcp_tool_executions p WHERE p.mcp_execution_id::text = x.mcp_execution_id::text)) AS orphans FROM plugin_usage_events x
UNION ALL
SELECT 'plugin_usage_events.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM plugin_usage_events x
UNION ALL
SELECT 'plugin_usage_events.tool_use_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.tool_use_id IS NULL) AS nulls, count(*) FILTER (WHERE x.tool_use_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_request_tool_calls p WHERE p.ai_tool_call_id::text = x.tool_use_id::text)) AS orphans FROM plugin_usage_events x
UNION ALL
SELECT 'plugin_usage_events.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM plugin_usage_events x
UNION ALL
SELECT 'project_ad_mappings.project_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.project_id IS NULL) AS nulls, count(*) FILTER (WHERE x.project_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM projects p WHERE p.id::text = x.project_id::text)) AS orphans FROM project_ad_mappings x
UNION ALL
SELECT 'project_members.project_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.project_id IS NULL) AS nulls, count(*) FILTER (WHERE x.project_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM projects p WHERE p.id::text = x.project_id::text)) AS orphans FROM project_members x
UNION ALL
SELECT 'project_members.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM project_members x
UNION ALL
SELECT 'salesforce_user_identities.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM salesforce_user_identities x
UNION ALL
SELECT 'secret_audit_log.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM secret_audit_log x
UNION ALL
SELECT 'secret_resolution_tokens.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM secret_resolution_tokens x
UNION ALL
SELECT 'session_cost_snapshots.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM session_cost_snapshots x
UNION ALL
SELECT 'session_cost_snapshots.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM session_cost_snapshots x
UNION ALL
SELECT 'session_entity_links.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM session_entity_links x
UNION ALL
SELECT 'session_entity_links.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM session_entity_links x
UNION ALL
SELECT 'session_ratings.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM session_ratings x
UNION ALL
SELECT 'session_ratings.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM session_ratings x
UNION ALL
SELECT 'session_transcripts.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM session_transcripts x
UNION ALL
SELECT 'session_transcripts.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM session_transcripts x
UNION ALL
SELECT 'skill_invocation_events.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM skill_invocation_events x
UNION ALL
SELECT 'skill_invocation_events.tool_use_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.tool_use_id IS NULL) AS nulls, count(*) FILTER (WHERE x.tool_use_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_request_tool_calls p WHERE p.ai_tool_call_id::text = x.tool_use_id::text)) AS orphans FROM skill_invocation_events x
UNION ALL
SELECT 'skill_invocation_events.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM skill_invocation_events x
UNION ALL
SELECT 'skill_ratings.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM skill_ratings x
UNION ALL
SELECT 'task_artifacts.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM task_artifacts x
UNION ALL
SELECT 'task_artifacts.mcp_execution_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.mcp_execution_id IS NULL) AS nulls, count(*) FILTER (WHERE x.mcp_execution_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM mcp_tool_executions p WHERE p.mcp_execution_id::text = x.mcp_execution_id::text)) AS orphans FROM task_artifacts x
UNION ALL
SELECT 'task_artifacts.task_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.task_id IS NULL) AS nulls, count(*) FILTER (WHERE x.task_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM agent_tasks p WHERE p.task_id::text = x.task_id::text)) AS orphans FROM task_artifacts x
UNION ALL
SELECT 'task_execution_steps.task_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.task_id IS NULL) AS nulls, count(*) FILTER (WHERE x.task_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM agent_tasks p WHERE p.task_id::text = x.task_id::text)) AS orphans FROM task_execution_steps x
UNION ALL
SELECT 'task_messages.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM task_messages x
UNION ALL
SELECT 'task_messages.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM task_messages x
UNION ALL
SELECT 'task_messages.task_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.task_id IS NULL) AS nulls, count(*) FILTER (WHERE x.task_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM agent_tasks p WHERE p.task_id::text = x.task_id::text)) AS orphans FROM task_messages x
UNION ALL
SELECT 'task_messages.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM task_messages x
UNION ALL
SELECT 'tenant_activity.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM tenant_activity x
UNION ALL
SELECT 'tool_activity.ai_tool_call_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.ai_tool_call_id IS NULL) AS nulls, count(*) FILTER (WHERE x.ai_tool_call_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_request_tool_calls p WHERE p.ai_tool_call_id::text = x.ai_tool_call_id::text)) AS orphans FROM tool_activity x
UNION ALL
SELECT 'tool_activity.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM tool_activity x
UNION ALL
SELECT 'tool_activity.mcp_execution_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.mcp_execution_id IS NULL) AS nulls, count(*) FILTER (WHERE x.mcp_execution_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM mcp_tool_executions p WHERE p.mcp_execution_id::text = x.mcp_execution_id::text)) AS orphans FROM tool_activity x
UNION ALL
SELECT 'tool_activity.request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.request_id::text)) AS orphans FROM tool_activity x
UNION ALL
SELECT 'tool_activity.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM tool_activity x
UNION ALL
SELECT 'tool_activity.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM tool_activity x
UNION ALL
SELECT 'tool_call_ledger.ai_tool_call_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.ai_tool_call_id IS NULL) AS nulls, count(*) FILTER (WHERE x.ai_tool_call_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_request_tool_calls p WHERE p.ai_tool_call_id::text = x.ai_tool_call_id::text)) AS orphans FROM tool_call_ledger x
UNION ALL
SELECT 'tool_call_ledger.context_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.context_id IS NULL) AS nulls, count(*) FILTER (WHERE x.context_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_contexts p WHERE p.context_id::text = x.context_id::text)) AS orphans FROM tool_call_ledger x
UNION ALL
SELECT 'tool_call_ledger.mcp_execution_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.mcp_execution_id IS NULL) AS nulls, count(*) FILTER (WHERE x.mcp_execution_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM mcp_tool_executions p WHERE p.mcp_execution_id::text = x.mcp_execution_id::text)) AS orphans FROM tool_call_ledger x
UNION ALL
SELECT 'tool_call_ledger.request_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.request_id IS NULL) AS nulls, count(*) FILTER (WHERE x.request_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests p WHERE p.id::text = x.request_id::text)) AS orphans FROM tool_call_ledger x
UNION ALL
SELECT 'tool_call_ledger.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM tool_call_ledger x
UNION ALL
SELECT 'tool_call_ledger.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM tool_call_ledger x
UNION ALL
SELECT 'user_activity.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_activity x
UNION ALL
SELECT 'user_api_keys.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_api_keys x
UNION ALL
SELECT 'user_contexts.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM user_contexts x
UNION ALL
SELECT 'user_contexts.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_contexts x
UNION ALL
SELECT 'user_device_certs.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_device_certs x
UNION ALL
SELECT 'user_encryption_keys.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_encryption_keys x
UNION ALL
SELECT 'user_groups.group_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.group_id IS NULL) AS nulls, count(*) FILTER (WHERE x.group_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM groups p WHERE p.id::text = x.group_id::text)) AS orphans FROM user_groups x
UNION ALL
SELECT 'user_groups.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_groups x
UNION ALL
SELECT 'user_manual_roles.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_manual_roles x
UNION ALL
SELECT 'user_profile_ext.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_profile_ext x
UNION ALL
SELECT 'user_profile_reports.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_profile_reports x
UNION ALL
SELECT 'user_projects.project_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.project_id IS NULL) AS nulls, count(*) FILTER (WHERE x.project_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM projects p WHERE p.id::text = x.project_id::text)) AS orphans FROM user_projects x
UNION ALL
SELECT 'user_projects.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_projects x
UNION ALL
SELECT 'user_rate_limit_buckets.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_rate_limit_buckets x
UNION ALL
SELECT 'user_scope_defaults.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_scope_defaults x
UNION ALL
SELECT 'user_sessions.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_sessions x
UNION ALL
SELECT 'user_settings.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM user_settings x
UNION ALL
SELECT 'v_all_activity.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM v_all_activity x
UNION ALL
SELECT 'v_bot_sessions.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM v_bot_sessions x
UNION ALL
SELECT 'v_bot_sessions.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM v_bot_sessions x
UNION ALL
SELECT 'v_clean_traffic.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM v_clean_traffic x
UNION ALL
SELECT 'v_clean_traffic.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM v_clean_traffic x
UNION ALL
SELECT 'v_engaged_traffic.session_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.session_id IS NULL) AS nulls, count(*) FILTER (WHERE x.session_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM user_sessions p WHERE p.session_id::text = x.session_id::text)) AS orphans FROM v_engaged_traffic x
UNION ALL
SELECT 'v_engaged_traffic.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM v_engaged_traffic x
UNION ALL
SELECT 'webauthn_challenges.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM webauthn_challenges x
UNION ALL
SELECT 'webauthn_credentials.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM webauthn_credentials x
UNION ALL
SELECT 'webauthn_setup_tokens.user_id' AS col, count(*) AS rows, count(*) FILTER (WHERE x.user_id IS NULL) AS nulls, count(*) FILTER (WHERE x.user_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM users p WHERE p.id::text = x.user_id::text)) AS orphans FROM webauthn_setup_tokens x
) t WHERE orphans > 0 OR col LIKE ANY (ARRAY['ai_requests.%','mcp_tool_executions.%','governance_decisions.%','plugin_usage_events.%','ai_request_tool_calls.%','logs.%','analytics_events.%','user_contexts.%','agent_tasks.%']) ORDER BY orphans DESC, col;
