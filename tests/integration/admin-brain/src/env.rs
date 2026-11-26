//! A Brain with a corpus, and the admin's routes over it.

use std::sync::Arc;

use async_trait::async_trait;
use axum::Extension;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use dynapps_test_common::TempDb;
use dynbrain::BrainRepository;
use dynbrain::access::{Grant, Level, Subject};
use dynbrain::brain::Brain;
use dynbrain::brain::native::NativeBrain;
use dynbrain::config::Mode;
use dynbrain::console::{Console, Native};
use dynbrain::ids::SpaceId;
use dynbrain::ingest::provider::{DIMENSIONS, Embedder, Embedding, Failure};
use dynbrain::model::{Article, NewArticle, NewSpace, RagMode, Space, VariantWrite};
use dynbrain::properties::PropertySchema;
use dynbrain::search::Retriever;
use dynbrain::search::query::EmbedderSlot;
use serde_json::{Map, json};
use systemprompt::identifiers::{Email, UserId};
use systemprompt_web_admin_brain::routes::routes_with;
use systemprompt_web_admin_brain::source::Source;
use systemprompt_web_admin_core::repositories::scope::Participation;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};
use systemprompt_web_shared::RankTier;
use tower::ServiceExt;

pub(crate) const FINGERPRINT: &str = "test:query:1536";

pub(crate) fn user(name: &str) -> UserId {
    UserId::new(name.to_owned())
}

pub(crate) fn axis(hot: usize) -> String {
    let parts: Vec<String> = (0..DIMENSIONS)
        .map(|at| if at == hot { "1" } else { "0" }.to_owned())
        .collect();
    format!("[{}]", parts.join(","))
}

struct AxisEmbedder;

#[async_trait]
impl Embedder for AxisEmbedder {
    fn fingerprint(&self) -> String {
        FINGERPRINT.to_owned()
    }

    async fn embed(&self, texts: &[String]) -> Result<Embedding, Failure> {
        let mut vector = vec![0.0; DIMENSIONS];
        vector[0] = 1.0;
        Ok(Embedding {
            vectors: texts.iter().map(|_| vector.clone()).collect(),
            input_tokens: 1,
            cost_microdollars: 0,
        })
    }
}

pub(crate) struct Reply {
    pub(crate) status: StatusCode,
    pub(crate) html: String,
    pub(crate) location: String,
}

impl Reply {
    pub(crate) fn has(&self, needle: &str) -> bool {
        self.html
            .replace("&#x3D;", "=")
            .replace("&#x27;", "'")
            .replace("&quot;", "\"")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .contains(needle)
    }

    pub(crate) fn has_fault_notice(&self) -> bool {
        self.has("Some of this page could not be loaded")
    }
}

pub(crate) struct Env {
    pub(crate) db: TempDb,
    pub(crate) repo: BrainRepository,
    pub(crate) native: Arc<NativeBrain>,
    pub(crate) retriever: Retriever,
    pub(crate) console: Console,
    pub(crate) invoicing: Article,
    pub(crate) credit: Article,
    pub(crate) sla: Article,
}

impl Env {
    // alice owns the spaces; root is an administrator; bob reads `handbook`
    // only; carol has no access to anything. `invoicing` has two chunks that
    // match "apple", its child `credit-notes` one chunk whose embedding
    // failed, and `sla` (in `support`) one more.
    pub(crate) async fn new(db: TempDb) -> Self {
        for (name, roles) in [
            ("alice", "user"),
            ("bob", "user"),
            ("carol", "user"),
            ("root", "admin"),
        ] {
            sqlx::query(
                "INSERT INTO users (id, name, email, roles, status) VALUES ($1, $1, $2, $3, 'active')",
            )
            .bind(name)
            .bind(format!("{name}@example.test"))
            .bind(vec![roles.to_owned()])
            .execute(&*db.pool)
            .await
            .expect("seed a user");
        }
        let repo = BrainRepository::new(Arc::clone(&db.pool));
        let handbook = space(&repo, "handbook").await;
        let support = space(&repo, "support").await;
        let grant = Grant {
            space: handbook.id,
            article: None,
            subject: Subject::user(&user("bob")),
            level: Level::Read,
        };
        repo.set_grant(&grant, &user("alice"))
            .await
            .expect("let bob read the handbook");
        let invoicing = article(&repo, handbook.id, None, "invoicing").await;
        let credit = article(&repo, handbook.id, Some(&invoicing), "credit-notes").await;
        let sla = article(&repo, support.id, None, "sla").await;
        write(&repo, &invoicing, "es", "Facturas", "Cómo emitir facturas.").await;
        write(
            &repo,
            &invoicing,
            "en",
            "Invoices",
            "How to issue invoices.",
        )
        .await;
        write(
            &repo,
            &credit,
            "es",
            "Rectificativas",
            "Corregir una factura.",
        )
        .await;
        write(&repo, &sla, "es", "Weekend SLA", "Support at the weekend.").await;
        let rows = [
            (
                &invoicing,
                0,
                "Facturas › Rectificativas",
                "apple credit notes to correct an invoice",
                Some(0),
            ),
            (
                &invoicing,
                1,
                "Facturas › Cierre",
                "apple closing the period",
                Some(1),
            ),
            (&sla, 0, "", "apple weekend support hours", Some(0)),
            (
                &credit,
                0,
                "Rectificativas",
                "pending credit note text",
                None,
            ),
        ];
        for (article, sequence, heading, content, hot) in rows {
            chunk(&db, article, (sequence, heading), content, hot).await;
        }
        let retriever =
            Retriever::new(repo.clone()).with_embedder(EmbedderSlot::ready(Arc::new(AxisEmbedder)));
        let native = Arc::new(NativeBrain::new(repo.clone()).with_search(retriever.clone()));
        let shared = Arc::clone(&native);
        let brain: Arc<dyn Brain> = shared;
        let console = Console {
            mode: Mode::Native,
            brain,
            native: Some(Native {
                brain: Arc::clone(&native),
                repo: repo.clone(),
                retriever: retriever.clone(),
            }),
        };
        Self {
            db,
            repo,
            native,
            retriever,
            console,
            invoicing,
            credit,
            sla,
        }
    }

    // The same instance as a proxy sees it: no index handle, and the tables
    // the technical pages would read are gone.
    pub(crate) async fn proxy(db: TempDb) -> Self {
        let mut env = Self::new(db).await;
        sqlx::query(
            "DROP TABLE dynbrain_chunk, dynbrain_index_job, dynbrain_search_log, dynbrain_embedding_usage CASCADE",
        )
        .execute(&*env.db.pool)
        .await
        .expect("drop the index tables");
        env.console.mode = Mode::Proxy;
        env.console.native = None;
        env
    }

    pub(crate) async fn get(&self, uri: &str, admin: bool) -> Reply {
        self.send(Request::builder().uri(uri).body(Body::empty()), admin)
            .await
    }

    pub(crate) async fn post(&self, uri: &str, form: &str, admin: bool) -> Reply {
        let request = Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(form.to_owned()));
        self.send(request, admin).await
    }

    async fn send(&self, request: Result<Request<Body>, axum::http::Error>, admin: bool) -> Reply {
        let app = routes_with::<()>(Source::Fixed(Box::new(self.console.clone())))
            .layer(Extension(engine()))
            .layer(Extension(viewer(admin)))
            .layer(Extension(marketplace()));
        let response = app
            .oneshot(request.expect("build the request"))
            .await
            .expect("answer the request");
        let status = response.status();
        let location = response
            .headers()
            .get("location")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        let bytes = axum::body::to_bytes(response.into_body(), 4 * 1024 * 1024)
            .await
            .expect("read the body");
        Reply {
            status,
            html: String::from_utf8_lossy(&bytes).into_owned(),
            location,
        }
    }

    pub(crate) async fn sql_text(&self, query: &'static str) -> String {
        sqlx::query_scalar::<_, String>(query)
            .fetch_one(&*self.db.pool)
            .await
            .expect("read a value")
    }

    pub(crate) async fn sql_for(&self, query: &'static str, id: i64) -> String {
        sqlx::query_scalar::<_, String>(query)
            .bind(id)
            .fetch_one(&*self.db.pool)
            .await
            .expect("read a value")
    }

    pub(crate) async fn finish(self) {
        self.db.cleanup().await;
    }
}

async fn space(repo: &BrainRepository, slug: &str) -> Space {
    repo.insert_space(&NewSpace {
        slug: slug.to_owned(),
        name: format!("Space {slug}"),
        default_lang: "es".to_owned(),
        langs: vec!["es".to_owned(), "en".to_owned()],
        properties: PropertySchema::parse(&json!([])).expect("an empty schema"),
        created_by: user("alice"),
    })
    .await
    .expect("create a space")
}

async fn article(
    repo: &BrainRepository,
    space: SpaceId,
    parent: Option<&Article>,
    slug: &str,
) -> Article {
    repo.insert_article(
        &NewArticle {
            space,
            parent: parent.map(|parent| parent.id),
            position: None,
            slug: slug.to_owned(),
            icon: None,
            rag_mode: RagMode::Default,
        },
        &user("alice"),
    )
    .await
    .expect("create an article")
}

async fn write(repo: &BrainRepository, article: &Article, lang: &str, title: &str, body: &str) {
    let content = VariantWrite {
        title: title.to_owned(),
        description: String::new(),
        body: body.to_owned(),
        front_matter: Map::new(),
        properties: Map::new(),
    };
    repo.write_variant(article.id, lang, &content, &user("alice"))
        .await
        .expect("write a variant");
}

async fn chunk(
    db: &TempDb,
    article: &Article,
    (sequence, heading): (i32, &str),
    content: &str,
    hot: Option<usize>,
) {
    sqlx::query(
        "INSERT INTO dynbrain_chunk
                (article_id, lang, sequence, heading_path, content, content_hash, fts_config,
                 embedding, embedding_fingerprint, embedded_at, has_error)
         VALUES ($1, 'es', $2, $3, $4, md5($4), 'simple'::regconfig, $5::text::vector,
                 CASE WHEN $5::text IS NULL THEN NULL ELSE $6 END,
                 CASE WHEN $5::text IS NULL THEN NULL ELSE CURRENT_TIMESTAMP END,
                 $5::text IS NULL)",
    )
    .bind(article.id.0)
    .bind(sequence)
    .bind(heading)
    .bind(content)
    .bind(hot.map(axis))
    .bind(FINGERPRINT)
    .execute(&*db.pool)
    .await
    .expect("index a chunk");
}

fn engine() -> AdminTemplateEngine {
    let admin =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../storage/files/admin");
    AdminTemplateEngine::new(&admin, &admin).expect("load the admin templates")
}

fn viewer(admin: bool) -> UserContext {
    UserContext {
        user_id: user(if admin { "root" } else { "bob" }),
        username: "viewer".to_owned(),
        email: Email::try_new("viewer@example.com").expect("an email"),
        roles: vec![if admin { "admin" } else { "user" }.to_owned()],
        group_ids: Vec::new(),
        project_ids: Vec::new(),
        participation: Participation::default(),
        is_admin: admin,
        is_console: admin,
        is_platform_admin: admin,
        is_developer: false,
        email_verified: true,
        session_id: None,
        browser_sign_in_at: None,
    }
}

fn marketplace() -> MarketplaceContext {
    MarketplaceContext {
        user_id: user("root"),
        site_url: "https://governance.invalid".to_owned(),
        total_plugins: 0,
        total_skills: 0,
        agents_count: 0,
        mcp_count: 0,
        rank_level: 0,
        rank_name: String::new(),
        rank_tier: RankTier::default(),
        total_xp: 0,
        xp_progress_pct: 0.0,
        has_completed_onboarding: true,
        current_streak: 0,
        longest_streak: 0,
        next_rank_name: String::new(),
        xp_to_next_rank: 0,
    }
}
