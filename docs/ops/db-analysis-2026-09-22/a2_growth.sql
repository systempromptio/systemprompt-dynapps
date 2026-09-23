-- A2: per-table first/last row, rows in last 90d, per-day rate, 12-month projection using bytes/row
DO $$ DECLARE r record; first_at timestamptz; last_at timestamptz; n90 bigint; n7 bigint; total bigint; BEGIN
  CREATE TEMP TABLE IF NOT EXISTS _growth(tbl text primary key, ts_col text, first_at timestamptz, last_at timestamptz, rows_total bigint, rows_90d bigint, rows_7d bigint, total_bytes bigint);
  FOR r IN
    SELECT DISTINCT ON (c.table_schema, c.table_name) c.table_schema s, c.table_name t, c.column_name col
    FROM information_schema.columns c JOIN pg_tables p ON p.schemaname=c.table_schema AND p.tablename=c.table_name
    WHERE c.table_schema IN ('public','marketplace') AND c.data_type LIKE 'timestamp%'
      AND c.column_name IN ('created_at','occurred_at','timestamp','started_at','recorded_at','captured_at','accepted_at','logged_at','applied_at','generated_at','issued_at','received_at','bound_at','requested_at','updated_at')
    ORDER BY c.table_schema, c.table_name, array_position(ARRAY['created_at','occurred_at','timestamp','started_at','recorded_at','captured_at','accepted_at','logged_at','applied_at','generated_at','issued_at','received_at','bound_at','requested_at','updated_at'], c.column_name::text)
  LOOP
    EXECUTE format('SELECT min(%I), max(%I), count(*) FILTER (WHERE %I > now()-interval ''90 days''), count(*) FILTER (WHERE %I > now()-interval ''7 days''), count(*) FROM %I.%I', r.col, r.col, r.col, r.col, r.s, r.t) INTO first_at, last_at, n90, n7, total;
    INSERT INTO _growth VALUES (r.s||'.'||r.t, r.col, first_at, last_at, total, n90, n7, pg_total_relation_size(format('%I.%I', r.s, r.t)::regclass)) ON CONFLICT (tbl) DO NOTHING;
  END LOOP; END $$;
SELECT tbl, ts_col, first_at::date AS first, last_at::date AS last, rows_total, rows_90d, rows_7d,
       round(rows_90d/90.0,1) AS per_day_90, round(rows_7d/7.0,1) AS per_day_7,
       pg_size_pretty(total_bytes) AS now_size,
       CASE WHEN rows_total>0 THEN pg_size_pretty((total_bytes::numeric/rows_total*(rows_total + GREATEST(rows_90d/90.0, rows_7d/7.0)*365))::bigint) END AS proj_12m
FROM _growth WHERE rows_total>0 ORDER BY total_bytes DESC LIMIT 40;
