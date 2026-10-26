//! Resolving the access matrix for a subject that is not necessarily a person.
//!
//! The resolver takes a user id, a role list and a bag of attribute values. A
//! *group* has none of those, but the audience matrix on the marketplaces page
//! asks exactly the same question of a group that the per-user matrix asks of a
//! person: which entities does this subject reach, and which band decided?
//!
//! [`MatrixSubject`] is that question's subject. A real user carries their own
//! id, roles and attributes; a synthetic group or role subject carries a
//! placeholder id and exactly one attribute band, so a `group` rule fires for
//! it and nothing else does. The placeholder id can never collide with a real
//! account: `UserId` values are emails or directory identifiers, and `group:`
//! is not a legal prefix for either.

use sqlx::PgPool;
use systemprompt::governance::authz::{RuleType, SubjectAttributes};
use systemprompt::identifiers::UserId;

use super::matrix::{
    MatrixSection, SectionInput, resolution_inputs, resolve_sections_for, sections_with,
};

#[derive(Debug, Clone)]
pub struct MatrixSubject {
    pub id: UserId,
    pub roles: Vec<String>,
    pub attributes: SubjectAttributes,
}

fn banded_subject(prefix: &str, band: &str, value: &str) -> MatrixSubject {
    let mut attributes = SubjectAttributes::new();
    if let Ok(rule_type) = RuleType::extension(band.to_owned()) {
        attributes.insert(rule_type, vec![value.to_owned()]);
    }
    MatrixSubject {
        id: UserId::new(format!("{prefix}:{value}")),
        roles: Vec::new(),
        attributes,
    }
}

#[must_use]
pub fn group_subject(slug: &str) -> MatrixSubject {
    banded_subject("group", "group", slug)
}

#[must_use]
pub fn project_subject(slug: &str) -> MatrixSubject {
    banded_subject("project", "project", slug)
}

#[must_use]
pub fn role_subject(role: &str) -> MatrixSubject {
    MatrixSubject {
        id: UserId::new(format!("role:{role}")),
        roles: vec![role.to_owned()],
        attributes: SubjectAttributes::new(),
    }
}

pub async fn user_subject(
    services: &crate::extension_services::ExtensionServices,
    user_id: &UserId,
    roles: Vec<String>,
) -> Result<MatrixSubject, crate::authz::AuthzReadError> {
    Ok(MatrixSubject {
        id: user_id.clone(),
        roles,
        attributes: services.subject_attributes(user_id).await?,
    })
}

pub async fn resolve_subject_matrix(
    pool: &PgPool,
    services: &crate::extension_services::ExtensionServices,
    subject: &MatrixSubject,
    sections: Vec<SectionInput>,
) -> Result<Vec<MatrixSection>, crate::authz::AuthzReadError> {
    resolve_sections_for(pool, services, subject, sections).await
}

pub async fn resolve_subject_matrices(
    pool: &PgPool,
    services: &crate::extension_services::ExtensionServices,
    subjects: &[MatrixSubject],
    sections: &[SectionInput],
) -> Result<Vec<Vec<MatrixSection>>, crate::authz::AuthzReadError> {
    let inputs = resolution_inputs(pool, services).await?;
    let dimensions = (services).dimensions();
    Ok(subjects
        .iter()
        .map(|subject| sections_with(&inputs, dimensions, subject, sections.to_vec()))
        .collect())
}
