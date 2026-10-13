//! The `farm` subject dimension: its name and rung, and that it is
//! header-only, so a request without the header carries no farm.

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use systemprompt_security::authz::SubjectAttributeProvider;
use systemprompt_web_admin_farm::attribution::{
    FARM_DIMENSION, FarmAttributeProvider, farm_dimension,
};

fn provider() -> FarmAttributeProvider {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused@localhost/unused")
        .expect("a lazy pool never connects");
    FarmAttributeProvider::new(Arc::new(pool))
}

#[tokio::test]
async fn the_provider_names_the_farm_dimension() {
    let dimension = provider().dimension();
    assert_eq!(dimension.rule_type.as_str(), FARM_DIMENSION);
    assert_eq!(dimension.label, "Farm");
    assert_eq!(dimension.precedence, farm_dimension().precedence);
}

#[tokio::test]
async fn the_farm_dimension_is_header_only() {
    assert!(!provider().default_attribution());
}
