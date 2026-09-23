-- E30: deleted users with residual rows (users.status='deleted' count first)
SELECT status, count(*) FROM users GROUP BY 1;
-- also: user ids referenced by user-keyed tables that no longer exist in users at all (hard-deleted)
DO $$ DECLARE r record; n bigint; BEGIN
  CREATE TEMP TABLE IF NOT EXISTS _ghost(tbl text primary key, rows_for_missing_users bigint);
  FOR r IN SELECT table_name t FROM information_schema.columns WHERE table_schema='public' AND column_name='user_id' AND table_name NOT IN ('users') LOOP
    EXECUTE format('SELECT count(*) FROM %I t WHERE t.user_id IS NOT NULL AND t.user_id::text NOT IN (''system'') AND NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id::text)', r.t) INTO n;
    INSERT INTO _ghost VALUES (r.t, n) ON CONFLICT (tbl) DO UPDATE SET rows_for_missing_users=EXCLUDED.rows_for_missing_users;
  END LOOP; END $$;
SELECT * FROM _ghost WHERE rows_for_missing_users>0 ORDER BY 2 DESC;
SELECT count(*) user_keyed_tables FROM _ghost;
-- E31: oldest row per table vs retention
SELECT tbl, ts_col, first_at::date oldest, now()::date - first_at::date age_days, rows_total FROM _growth WHERE rows_total>0 ORDER BY first_at LIMIT 40;
