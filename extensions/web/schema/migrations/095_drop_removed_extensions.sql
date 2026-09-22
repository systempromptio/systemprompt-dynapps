-- Tables that no crate declares any more. The knowledge-bank and
-- requirements-workflow MCP servers were removed in 348b8efb and the
-- session-evaluation extension in c791ad5a, each without a drop migration,
-- so their tables, one scheduled_jobs row and the `evaluation` migration
-- ledger survived on every installed instance (39 undeclared tables on
-- production, 2026-09-22 dump). Core's managed_resources_011 lists the
-- eval_* engine tables, but on production it was baseline-stamped as
-- applied (2026-09-21) without executing, so every eval_* table survived
-- and the ledger says there is nothing left to run; the full set is dropped
-- here. All rows were zero on production. The employee_* gamification tables
-- (08_gamification.sql, removed in 653fcef8) never reached production but
-- may exist on an older install. CASCADE takes dependent views, triggers
-- and foreign keys. Idempotent by IF EXISTS.
DROP TABLE IF EXISTS knowledge_chunks CASCADE;
DROP TABLE IF EXISTS knowledge_date_cache CASCADE;
DROP TABLE IF EXISTS knowledge_document_grants CASCADE;
DROP TABLE IF EXISTS knowledge_documents CASCADE;
DROP TABLE IF EXISTS knowledge_drive_document_projects CASCADE;
DROP TABLE IF EXISTS knowledge_drive_files CASCADE;
DROP TABLE IF EXISTS knowledge_drive_sources CASCADE;
DROP TABLE IF EXISTS knowledge_embedding_cache CASCADE;
DROP TABLE IF EXISTS knowledge_index_state CASCADE;
DROP TABLE IF EXISTS knowledge_source_text CASCADE;
DROP TABLE IF EXISTS knowledge_sync_runs CASCADE;
DROP TABLE IF EXISTS knowledge_user_sync_state CASCADE;
DROP TABLE IF EXISTS requirements_publication_attempts CASCADE;
DROP TABLE IF EXISTS requirements_validation_jobs CASCADE;
DROP TABLE IF EXISTS requirements_workflow_events CASCADE;
DROP TABLE IF EXISTS requirements_workflow_keys CASCADE;
DROP TABLE IF EXISTS requirements_workflow_review_nonces CASCADE;
DROP TABLE IF EXISTS requirements_workflows CASCADE;
DROP TABLE IF EXISTS eval_budget_accounts CASCADE;
DROP TABLE IF EXISTS eval_budget_reservations CASCADE;
DROP TABLE IF EXISTS eval_cases CASCADE;
DROP TABLE IF EXISTS eval_execution_artifacts CASCADE;
DROP TABLE IF EXISTS eval_execution_capabilities CASCADE;
DROP TABLE IF EXISTS eval_execution_events CASCADE;
DROP TABLE IF EXISTS eval_execution_evidence CASCADE;
DROP TABLE IF EXISTS eval_executions CASCADE;
DROP TABLE IF EXISTS eval_experiments CASCADE;
DROP TABLE IF EXISTS eval_frozen_workspaces CASCADE;
DROP TABLE IF EXISTS eval_judge_calls CASCADE;
DROP TABLE IF EXISTS eval_pairs CASCADE;
DROP TABLE IF EXISTS eval_request_reservations CASCADE;
DROP TABLE IF EXISTS eval_resource_revisions CASCADE;
DROP TABLE IF EXISTS eval_results CASCADE;
DROP TABLE IF EXISTS eval_rubrics CASCADE;
DROP TABLE IF EXISTS eval_runs CASCADE;
DROP TABLE IF EXISTS eval_session_bindings CASCADE;
DROP TABLE IF EXISTS eval_workers CASCADE;
DROP TABLE IF EXISTS employee_achievements CASCADE;
DROP TABLE IF EXISTS employee_daily_usage CASCADE;
DROP TABLE IF EXISTS employee_ranks CASCADE;
DROP TABLE IF EXISTS employee_xp_ledger CASCADE;
DELETE FROM scheduled_jobs WHERE job_name = 'knowledge_drive_sync';
DELETE FROM extension_migrations WHERE extension_id = 'evaluation';
