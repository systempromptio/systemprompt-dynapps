DO $$ DECLARE r record; total bigint; orphans bigint; ex text; parent text; ppk text; has_fk bool; BEGIN
  CREATE TEMP TABLE IF NOT EXISTS _orphans(tbl text, col text, parent text, rows_nonnull bigint, orphans bigint, has_fk bool, examples text);
  DELETE FROM _orphans;
  FOR r IN SELECT c.table_name t, c.column_name col FROM information_schema.columns c JOIN pg_tables p ON p.tablename=c.table_name AND p.schemaname='public'
           WHERE c.table_schema='public' AND c.column_name IN ('user_id','session_id','context_id','request_id','ai_request_id','task_id','group_id','project_id','owner_id') ORDER BY 1,2 LOOP
    parent := CASE r.col WHEN 'user_id' THEN 'users' WHEN 'owner_id' THEN 'users' WHEN 'session_id' THEN 'user_sessions' WHEN 'context_id' THEN 'user_contexts'
                         WHEN 'request_id' THEN 'ai_requests' WHEN 'ai_request_id' THEN 'ai_requests' WHEN 'task_id' THEN 'agent_tasks' WHEN 'group_id' THEN 'groups' WHEN 'project_id' THEN 'projects' END;
    IF r.t = parent THEN CONTINUE; END IF;
    ppk := CASE parent WHEN 'user_sessions' THEN 'session_id' WHEN 'user_contexts' THEN 'context_id' WHEN 'agent_tasks' THEN 'task_id' ELSE 'id' END;
    SELECT EXISTS(SELECT 1 FROM pg_constraint k WHERE k.contype='f' AND k.conrelid=('public.'||r.t)::regclass AND k.confrelid=('public.'||parent)::regclass
                  AND r.col IN (SELECT attname FROM pg_attribute WHERE attrelid=k.conrelid AND attnum = ANY(k.conkey))) INTO has_fk;
    EXECUTE format('SELECT count(*) FILTER (WHERE t.%I IS NOT NULL), count(*) FILTER (WHERE t.%I IS NOT NULL AND p.%I IS NULL), left(string_agg(DISTINCT t.%I::text, '','') FILTER (WHERE p.%I IS NULL AND t.%I IS NOT NULL),120) FROM %I t LEFT JOIN %I p ON p.%I::text = t.%I::text', r.col, r.col, ppk, r.col, ppk, r.col, r.t, parent, ppk, r.col) INTO total, orphans, ex;
    INSERT INTO _orphans VALUES (r.t, r.col, parent, total, orphans, has_fk, ex);
  END LOOP; END $$;
SELECT tbl, col, parent, rows_nonnull, orphans, round(100.0*orphans/NULLIF(rows_nonnull,0),1) pct, has_fk, examples FROM _orphans WHERE orphans>0 ORDER BY orphans DESC;
SELECT count(*) cols_checked, count(*) FILTER (WHERE has_fk) with_fk, count(*) FILTER (WHERE orphans>0) with_orphans FROM _orphans;
