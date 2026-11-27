//! The repository over a throwaway database, and the people it is used by.

use std::sync::Arc;

use dynapps_test_common::TempDb;
use dynbrain::BrainRepository;
use dynbrain::access::{Grant, Level, Subject};
use dynbrain::ids::{ArticleId, SpaceId};
use dynbrain::model::{Article, NewArticle, NewSpace, RagMode, Space, VariantWrite};
use dynbrain::properties::PropertySchema;
use serde_json::{Map, json};
use systemprompt::identifiers::UserId;

pub(crate) struct Fixture {
    pub(crate) repo: BrainRepository,
    db: TempDb,
}

pub(crate) fn user(name: &str) -> UserId {
    UserId::new(name.to_owned())
}

impl Fixture {
    // alice owns the spaces; root is an administrator; carol holds the
    // `editor` role; erin is in the `team` group and frank was, until his
    // membership was revoked; dave's account is suspended.
    pub(crate) async fn new(db: TempDb) -> Self {
        let people: [(&str, &[&str], &str); 7] = [
            ("alice", &["user"], "active"),
            ("bob", &["user"], "active"),
            ("carol", &["user", "editor"], "active"),
            ("dave", &["user", "editor"], "suspended"),
            ("erin", &["user"], "active"),
            ("frank", &["user"], "active"),
            ("root", &["admin"], "active"),
        ];
        for (name, roles, status) in people {
            let roles: Vec<String> = roles.iter().map(|role| (*role).to_owned()).collect();
            sqlx::query(
                "INSERT INTO users (id, name, email, roles, status) VALUES ($1, $1, $2, $3, $4)",
            )
            .bind(name)
            .bind(format!("{name}@example.test"))
            .bind(roles)
            .bind(status)
            .execute(&*db.pool)
            .await
            .expect("seed a user");
        }
        sqlx::query("INSERT INTO groups (id, name) VALUES ('team', 'Team')")
            .execute(&*db.pool)
            .await
            .expect("seed a group");
        sqlx::query(
            "INSERT INTO group_members (group_id, user_id, source, revoked_at)
             VALUES ('team', 'erin', 'manual', NULL),
                    ('team', 'frank', 'manual', CURRENT_TIMESTAMP - INTERVAL '1 day')",
        )
        .execute(&*db.pool)
        .await
        .expect("seed the group's members");
        Self {
            repo: BrainRepository::new(Arc::clone(&db.pool)),
            db,
        }
    }

    // Each test drops its own database: they share one Postgres.
    pub(crate) async fn finish(self) {
        self.db.cleanup().await;
    }

    pub(crate) fn pool(&self) -> &sqlx::PgPool {
        &self.db.pool
    }

    pub(crate) fn shared_pool(&self) -> Arc<sqlx::PgPool> {
        Arc::clone(&self.db.pool)
    }

    pub(crate) async fn give(
        &self,
        space: SpaceId,
        article: Option<ArticleId>,
        subject: Subject,
        level: Level,
    ) {
        let grant = Grant {
            space,
            article,
            subject,
            level,
        };
        self.repo
            .set_grant(&grant, &user("alice"))
            .await
            .expect("write the grant");
    }

    pub(crate) async fn write(&self, article: &Article, lang: &str, title: &str, body: &str) {
        self.repo
            .write_variant(article.id, lang, &content(title, body), &user("alice"))
            .await
            .expect("write a variant");
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
    ) -> Article {
        self.repo
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
            .expect("create an article")
    }
}

pub(crate) fn content(title: &str, body: &str) -> VariantWrite {
    VariantWrite {
        title: title.to_owned(),
        description: String::new(),
        body: body.to_owned(),
        front_matter: Map::new(),
        properties: Map::new(),
    }
}
