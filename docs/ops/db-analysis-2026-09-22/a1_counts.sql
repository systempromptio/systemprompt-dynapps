-- A1b: exact row counts for every table
DO $$ DECLARE r record; n bigint; BEGIN
  CREATE TEMP TABLE IF NOT EXISTS _rowcounts(tbl text primary key, rows bigint);
  FOR r IN SELECT schemaname s, tablename t FROM pg_tables WHERE schemaname IN ('public','marketplace') LOOP
    EXECUTE format('SELECT count(*) FROM %I.%I', r.s, r.t) INTO n;
    INSERT INTO _rowcounts VALUES (r.s||'.'||r.t, n) ON CONFLICT (tbl) DO UPDATE SET rows=EXCLUDED.rows;
  END LOOP; END $$;
SELECT * FROM _rowcounts ORDER BY rows DESC;
