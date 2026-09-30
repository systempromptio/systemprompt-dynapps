//! The role filter options for the user index listing.
//!
//! The role filter always offers the built-in assignable roles, even before
//! anyone holds them.

use sqlx::PgPool;

const SYSTEM_ROLES: [&str; 4] = ["anonymous", "a2a", "mcp", "service"];

const ASSIGNABLE_ROLES: [&str; 3] = [
    systemprompt_web_admin_core::types::ROLE_ADMIN,
    systemprompt_web_admin_core::types::ROLE_USER,
    systemprompt_web_admin_core::types::ROLE_PROJECT_MANAGER,
];

pub async fn list_distinct_roles(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT DISTINCT unnest(roles) AS "role!" FROM users
          WHERE NOT ('anonymous' = ANY(roles))
          ORDER BY 1"#,
    )
    .fetch_all(pool)
    .await?;
    let mut roles: Vec<String> = rows
        .into_iter()
        .map(|r| r.role)
        .filter(|r| !SYSTEM_ROLES.contains(&r.as_str()))
        .collect();
    for known in ASSIGNABLE_ROLES {
        if !roles.iter().any(|r| r == known) {
            roles.push(known.to_owned());
        }
    }
    roles.sort();
    Ok(roles)
}
