//! Throwaway database with the full extension schema applied.
//!
//! Each suite gets its own databases and template, prefixed by the running
//! test binary's name. The schema comes from `install_extension_schemas`, the
//! same path the server runs; migrations are collected from `inventory`
//! registrations, which exist only for crates linked into the binary --
//! hence the `use ... as _` below.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use sqlx::{AssertSqlSafe, PgPool};
use systemprompt::database::{Database, DbPool, MigrationConfig, install_extension_schemas_full};
use systemprompt::extension::ExtensionRegistry;
use url::Url;

use systemprompt_web_extension as _;

use crate::skip::skip_or_panic;

#[derive(Debug)]
pub struct TempDb {
    pub pool: Arc<PgPool>,
    admin_url: String,
    db_name: String,
}

// `None` lets a developer machine with no Postgres run the rest of the
// suite; under `CI` the absence is a panic. Only databases named `test`,
// `postgres` or `*_test` are accepted, so a stray `DATABASE_URL` pointing
// at a live dev database panics instead of being used.
fn server_url() -> Option<String> {
    let raw = std::env::var("SYSTEMPROMPT_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|u| !u.trim().is_empty());
    let Some(raw) = raw else {
        skip_or_panic(
            "SYSTEMPROMPT_TEST_DATABASE_URL",
            "DB-backed suites need a maintenance Postgres URL",
        );
        return None;
    };
    let parsed = Url::parse(&raw).expect("test database URL must be a valid URL");
    let db_name = parsed.path().trim_start_matches('/');
    let allowed = db_name == "test" || db_name == "postgres" || db_name.ends_with("_test");
    assert!(
        allowed,
        "Refusing to run integration tests against database '{db_name}'. Set \
         SYSTEMPROMPT_TEST_DATABASE_URL to a database reserved for tests: the database name \
         must be 'test', 'postgres', or end in '_test'."
    );
    Some(raw)
}

fn with_database(base: &str, db_name: &str) -> String {
    let mut url = Url::parse(base).expect("DATABASE_URL is a valid URL");
    url.set_path(&format!("/{db_name}"));
    url.into()
}

// Why: a schema install costs ~1s per test for an identical result, so each
// test copies a `CREATE DATABASE ... TEMPLATE`. nextest runs every test in
// its own process, so the cache must live in Postgres, keyed on the binary
// fingerprint (the schema is compiled in; a rebuild may over-report only).
fn template_name(prefix: &str) -> String {
    let exe = std::env::current_exe().expect("locate the running test binary");
    let meta = std::fs::metadata(&exe).expect("stat the running test binary");
    let mut hasher = DefaultHasher::new();
    meta.len().hash(&mut hasher);
    meta.modified().ok().hash(&mut hasher);
    format!("{prefix}_tpl_{:016x}", hasher.finish())
}

// Why: Cargo's build hash is dropped from the binary name; it changes every
// rebuild and the template already keys on the binary fingerprint.
fn suite_prefix() -> String {
    let exe = std::env::current_exe().expect("locate the running test binary");
    let stem = exe
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("dynapps_test")
        .to_owned();
    let stem = stem
        .rsplit_once('-')
        .filter(|(_, hash)| hash.len() == 16 && hash.chars().all(|c| c.is_ascii_hexdigit()))
        .map_or(stem.as_str(), |(head, _)| head);
    let cleaned: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    cleaned.chars().take(24).collect()
}

// Why: concurrent processes that find the template missing build it once
// under `pg_advisory_lock` rather than racing on `CREATE DATABASE`.
fn advisory_key(template: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    template.hash(&mut hasher);
    hasher.finish() as i64
}

async fn database_exists(conn: &mut sqlx::PgConnection, name: &str) -> bool {
    sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
        .bind(name)
        .fetch_one(conn)
        .await
        .expect("query pg_database")
}

// Why: Postgres `DROP DATABASE` errors on a database in use by a concurrent
// run, so failures are ignored; the next run collects the rest.
async fn drop_stale_templates(conn: &mut sqlx::PgConnection, prefix: &str, keep: &str) {
    let Ok(stale) = sqlx::query_scalar::<_, String>(
        // `LIKE` would treat the prefix's underscores as wildcards.
        "SELECT datname FROM pg_database WHERE left(datname, length($1)) = $1 AND datname <> $2",
    )
    .bind(format!("{prefix}_tpl_"))
    .bind(keep)
    .fetch_all(&mut *conn)
    .await
    else {
        return; // skip-ok: a stale template that cannot be listed is collected next run
    };
    for name in stale {
        sqlx::query(AssertSqlSafe(format!("DROP DATABASE IF EXISTS \"{name}\"")))
            .execute(&mut *conn)
            .await
            .ok();
    }
}

async fn ensure_template(admin: &PgPool, base: &str, prefix: &str, template: &str) {
    let mut conn = admin.acquire().await.expect("maintenance connection");
    // The advisory lock is session-scoped: every statement must run on this
    // one connection, or the unlock lands on a different session.
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(advisory_key(template))
        .execute(&mut *conn)
        .await
        .expect("take the template build lock");

    if !database_exists(&mut conn, template).await {
        sqlx::query(AssertSqlSafe(format!("CREATE DATABASE \"{template}\"")))
            .execute(&mut *conn)
            .await
            .expect("create the template database");
        let pool = Arc::new(
            PgPool::connect(&with_database(base, template))
                .await
                .expect("connect to the template database"),
        );
        let database = Database::from_pools(Arc::clone(&pool), Some(Arc::clone(&pool)));
        let registry = ExtensionRegistry::discover().expect("discover extension registrations");
        assert!(
            !registry.is_empty(),
            "no extensions registered — the test binary must link the crates whose \
             `register_extension!` supplies the migrations"
        );
        install_extension_schemas_full(
            &registry,
            database.write(),
            &[],
            MigrationConfig::default(),
        )
        .await
        .expect("install extension schemas");
        seed_test_projects(&pool).await;
        // Postgres refuses the template copy while any session holds it open.
        pool.close().await;
        drop_stale_templates(&mut conn, prefix, template).await;
    }

    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(advisory_key(template))
        .execute(&mut *conn)
        .await
        .expect("release the template build lock");
}

// Why: Postgres fails `CREATE DATABASE ... TEMPLATE` outright while any
// session is connected to the source, and a builder's closing connection
// can linger, so the copy is retried briefly.
async fn copy_template(admin: &PgPool, template: &str, db_name: &str) {
    let statement = format!("CREATE DATABASE \"{db_name}\" TEMPLATE \"{template}\"");
    let mut last = None;
    for _ in 0..50 {
        match sqlx::query(AssertSqlSafe(statement.clone()))
            .execute(admin)
            .await
        {
            Ok(_) => return,
            Err(e) => {
                last = Some(e);
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            },
        }
    }
    panic!(
        "could not copy the template database: {}",
        last.expect("a failure after the retry budget")
    );
}

// The two directory-backed projects the integration suites attach users to.
// Production projects come from `groups.yaml` through the boot sync, which
// the template install does not run, and the seeds carry no customer rows,
// so the suites own these fixtures.
async fn seed_test_projects(pool: &PgPool) {
    sqlx::query(
        "INSERT INTO projects (id, name, source)
         VALUES ('commerce', 'Commerce', 'yaml'), ('core', 'Core', 'yaml')
         ON CONFLICT (id) DO NOTHING",
    )
    .execute(pool)
    .await
    .expect("seed the suites' projects");
}

impl TempDb {
    pub async fn create() -> Option<Self> {
        let prefix = &suite_prefix();
        let base = server_url()?;
        // Postgres: CREATE DATABASE cannot run inside a transaction.
        let admin_url = with_database(&base, "postgres");
        let db_name = crate::orphans::database_name(prefix);

        let admin = PgPool::connect(&admin_url)
            .await
            .expect("connect to maintenance database");
        crate::orphans::sweep(&admin).await;
        let template = template_name(prefix);
        ensure_template(&admin, &base, prefix, &template).await;
        copy_template(&admin, &template, &db_name).await;
        admin.close().await;

        let pool = Arc::new(
            PgPool::connect(&with_database(&base, &db_name))
                .await
                .expect("connect to throwaway database"),
        );

        Some(Self {
            pool,
            admin_url,
            db_name,
        })
    }

    pub async fn create_empty() -> Option<Self> {
        let prefix = &suite_prefix();
        let base = server_url()?;
        let admin_url = with_database(&base, "postgres");
        let db_name = crate::orphans::database_name(prefix);

        let admin = PgPool::connect(&admin_url)
            .await
            .expect("connect to maintenance database");
        crate::orphans::sweep(&admin).await;
        sqlx::query(AssertSqlSafe(format!("CREATE DATABASE \"{db_name}\"")))
            .execute(&admin)
            .await
            .expect("create throwaway database");
        admin.close().await;

        let pool = Arc::new(
            PgPool::connect(&with_database(&base, &db_name))
                .await
                .expect("connect to throwaway database"),
        );

        Some(Self {
            pool,
            admin_url,
            db_name,
        })
    }

    #[must_use]
    pub fn url(&self) -> String {
        with_database(&self.admin_url, &self.db_name)
    }

    #[must_use]
    pub fn db_pool(&self) -> DbPool {
        Arc::new(Database::from_pools(Arc::clone(&self.pool), None))
    }

    pub async fn cleanup(self) {
        self.pool.close().await;
        let admin = PgPool::connect(&self.admin_url)
            .await
            .expect("reconnect maintenance database for drop");
        sqlx::query(AssertSqlSafe(format!(
            "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
            self.db_name
        )))
        .execute(&admin)
        .await
        .expect("drop throwaway database");
        admin.close().await;
    }
}
