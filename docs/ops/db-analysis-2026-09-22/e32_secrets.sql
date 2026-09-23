-- E32: secret shapes in text/jsonb columns — counts only. Patterns: sk-… keys, Bearer tokens, PEM private keys, JWT (three base64url segments with eyJ header)
DO $$ DECLARE r record; n_sk bigint; n_bearer bigint; n_pem bigint; n_jwt bigint; BEGIN
  CREATE TEMP TABLE IF NOT EXISTS _secrets(tbl text, col text, sk_ bigint, bearer bigint, pem bigint, jwt bigint);
  DELETE FROM _secrets;
  FOR r IN SELECT c.table_name t, c.column_name col, c.data_type dt FROM information_schema.columns c JOIN pg_tables p ON p.tablename=c.table_name AND p.schemaname='public'
           WHERE c.table_schema='public' AND c.data_type IN ('text','jsonb','json','character varying') AND p.tablename NOT LIKE 'analytics_report_%' LOOP
    EXECUTE format('SELECT count(*) FILTER (WHERE v ~ ''(^|[^A-Za-z0-9])sk-(ant-|proj-|live-|test-)?[A-Za-z0-9_-]{20,}''), count(*) FILTER (WHERE v ~ ''Bearer [A-Za-z0-9._~+/=-]{20,}''), count(*) FILTER (WHERE v ~ ''BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY''), count(*) FILTER (WHERE v ~ ''eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}'') FROM (SELECT %I::text v FROM %I WHERE %I IS NOT NULL) s', r.col, r.t, r.col) INTO n_sk, n_bearer, n_pem, n_jwt;
    IF n_sk+n_bearer+n_pem+n_jwt > 0 THEN INSERT INTO _secrets VALUES (r.t, r.col, n_sk, n_bearer, n_pem, n_jwt); END IF;
  END LOOP; END $$;
SELECT * FROM _secrets ORDER BY sk_+bearer+pem+jwt DESC;
