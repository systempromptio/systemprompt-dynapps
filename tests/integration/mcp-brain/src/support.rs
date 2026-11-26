//! The Brain over a throwaway database and the people it runs as.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use dynapps_test_common::TempDb;
use dynbrain::BrainRepository;
use dynbrain::access::{Grant, Level, Subject};
use dynbrain::assets::AssetFiles;
use dynbrain::assets::link::LinkSigner;
use dynbrain::brain::native::NativeBrain;
use dynbrain::ids::{ArticleId, SpaceId};
use dynbrain::model::{Article, NewArticle, NewSpace, RagMode, Space, VariantWrite};
use dynbrain::properties::PropertySchema;
use rmcp::ErrorData;
use serde_json::{Map, Value, json};
use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt::models::auth::{AuthenticatedUser, Permission};
use systemprompt::models::execution::context::RequestContext;
use systemprompt::storage::LocalFileStorage;
use systemprompt_mcp_brain::tools::run;

static ROOTS: AtomicU64 = AtomicU64::new(0);

pub(crate) struct Brain {
    pub(crate) native: NativeBrain,
    repo: BrainRepository,
    root: Option<PathBuf>,
    db: TempDb,
}

pub(crate) fn user(name: &str) -> UserId {
    UserId::new(name.to_owned())
}

// The context core hands a tool after authenticating the call.
pub(crate) fn person(name: &str) -> RequestContext {
    let id = user(name);
    anonymous().with_user(AuthenticatedUser::new(
        id,
        name.to_owned(),
        format!("{name}@example.test"),
        vec![Permission::User],
    ))
}

pub(crate) fn anonymous() -> RequestContext {
    RequestContext::new(
        SessionId::generate(),
        TraceId::new("trace".to_owned()),
        ContextId::generate(),
        AgentName::new("brain-tests"),
        Actor::user(user("nobody")),
    )
}

impl Brain {
    // alice owns the spaces and root is an administrator; bob and carol hold
    // nothing until a test grants it.
    pub(crate) async fn new(db: TempDb, with_files: bool) -> Self {
        for name in ["alice", "bob", "carol", "root"] {
            let roles: Vec<String> = vec![if name == "root" { "admin" } else { "user" }.to_owned()];
            sqlx::query(
                "INSERT INTO users (id, name, email, roles, status) VALUES ($1, $1, $2, $3, 'active')",
            )
            .bind(name)
            .bind(format!("{name}@example.test"))
            .bind(roles)
            .execute(&*db.pool)
            .await
            .expect("seed a user");
        }
        let repo = BrainRepository::new(Arc::clone(&db.pool));
        let root = with_files.then(|| {
            std::env::temp_dir().join(format!(
                "mcp-brain-files-{}-{}",
                std::process::id(),
                ROOTS.fetch_add(1, Ordering::Relaxed)
            ))
        });
        let files = root.as_ref().map(|root| {
            AssetFiles::new(
                Arc::new(LocalFileStorage::new(root.clone())),
                LinkSigner::new(&[9; 32]).expect("a link signer"),
                "https://brain.test".to_owned(),
            )
        });
        Self {
            native: NativeBrain::new(repo.clone()).with_files(files),
            repo,
            root,
            db,
        }
    }

    pub(crate) async fn finish(self) {
        if let Some(root) = &self.root {
            std::fs::remove_dir_all(root).ok();
        }
        self.db.cleanup().await;
    }

    pub(crate) async fn call(
        &self,
        who: &str,
        tool: &str,
        arguments: Value,
    ) -> Result<Value, ErrorData> {
        run(&self.native, &person(who), tool, arguments).await
    }

    pub(crate) async fn space(&self, slug: &str) -> Space {
        self.repo
            .insert_space(&NewSpace {
                slug: slug.to_owned(),
                name: format!("Space {slug}"),
                default_lang: "es".to_owned(),
                langs: vec!["es".to_owned(), "en".to_owned()],
                properties: PropertySchema::parse(&json!([
                    {"name": "status", "type": "selection", "selection": [["draft", "Draft"], ["decided", "Decided"]]}
                ]))
                .expect("a schema"),
                created_by: user("alice"),
            })
            .await
            .expect("create a space")
    }

    pub(crate) async fn article(
        &self,
        space: SpaceId,
        parent: Option<ArticleId>,
        slug: &str,
        (title, body): (&str, &str),
    ) -> Article {
        let article = self
            .repo
            .insert_article(
                &NewArticle {
                    space,
                    parent,
                    position: None,
                    slug: slug.to_owned(),
                    icon: None,
                    rag_mode: RagMode::Default,
                },
                &user("alice"),
            )
            .await
            .expect("create an article");
        self.repo
            .write_variant(
                article.id,
                "es",
                &VariantWrite {
                    title: title.to_owned(),
                    description: String::new(),
                    body: body.to_owned(),
                    front_matter: Map::new(),
                    properties: Map::new(),
                },
                &user("alice"),
            )
            .await
            .expect("write a variant");
        article
    }

    pub(crate) async fn give(
        &self,
        space: SpaceId,
        article: Option<ArticleId>,
        who: &str,
        level: Level,
    ) {
        self.repo
            .set_grant(
                &Grant {
                    space,
                    article,
                    subject: Subject::user(&user(who)),
                    level,
                },
                &user("alice"),
            )
            .await
            .expect("write the grant");
    }
}
