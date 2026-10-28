//! Web extension facade for the systemprompt template.
//!
//! Aggregates the web sibling crates (the admin crates, `content`, `jobs`,
//! `shared`, `site`) into a single `WebExtension` registered with the core
//! runtime. Most of this crate is re-exports — the actual extension entry
//! point is [`extension::WebExtension`], which advertises required assets,
//! HTTP routes, page data providers, and jobs to the host binary.
//!
//! The split between siblings is the layering boundary:
//!
//! - `admin` — the shared admin core and the five admin domain crates (catalog,
//!   governance, identity, platform, analytics), plus the routers this crate
//!   composes from them: SSR dashboard, governance webhooks, access-token
//!   plane.
//! - `content` — content ingestion, repositories, search, link analytics.
//! - `jobs` — `publish_pipeline` and its sub-jobs (asset copy, prerender,
//!   sitemap, llms.txt, secret migration, content analytics).
//! - `shared` — config schemas, error types, branding, ID newtypes, HTML-escape
//!   helpers.
//! - `site` — public homepage / docs content providers.

// Why: core finds external secrets sources through inventory, so the crate
// registering `onepassword` must be linked into the server binary or a
// profile naming that kind fails at boot.
use systemprompt_secrets_onepassword as _;
pub mod extension;
mod extension_impl;
mod router;
#[doc(hidden)]
pub mod schemas;

/// The admin plane: the shared admin core, the six domain crates, and the
/// routers composed from their fragments in `router::admin`.
pub mod admin {
    pub use crate::router::admin::{
        AdminSsrDeps, admin_router, admin_ssr_router, bridge_auth_ssr_router, drive_ssr_router,
        hooks_webhook_router, secrets_router, service_accounts, share_manifest_router,
    };
    pub use crate::router::farm_public::{build_state as build_farm_state, farm_public_router};
    pub use systemprompt_web_admin_analytics as analytics;
    pub use systemprompt_web_admin_catalog as catalog;
    pub use systemprompt_web_admin_core as core;
    pub use systemprompt_web_admin_farm as farm;
    pub use systemprompt_web_admin_governance as governance;
    pub use systemprompt_web_admin_identity as identity;
    pub use systemprompt_web_admin_platform as platform;
}
pub use systemprompt_web_content::{api, repository, services};
pub use systemprompt_web_jobs as jobs;
pub use systemprompt_web_shared as shared;
pub use systemprompt_web_shared::{
    BrandingConfig, config, config_errors, error, html_escape, models,
};
pub use systemprompt_web_site::{
    assets, docs, extenders, homepage, navigation, partials, skills_page,
};

pub use config::{
    BlogConfigRaw, BlogConfigValidated, ContentSourceRaw, ContentSourceValidated,
    ExtensionConfigError, ExtensionConfigErrors,
};
pub use docs::{ChildDoc, DocsContentDataProvider, DocsPageDataProvider};
pub use error::BlogError;
pub use extension::WebExtension;
pub use homepage::{
    ComparisonConfig, ComparisonItem, ComparisonSide, DifferentiatorConfig, DifferentiatorItem,
    ExtensionTrait, ExtensionsConfig, FaqConfig, FaqItem, Feature, FeatureCategory, FinalCtaConfig,
    HeroConfig, HomepageConfig, HomepageFeaturesConfig, HomepagePageDataProvider, HowItWorksConfig,
    HowItWorksStep, IntegrationBrand, IntegrationsConfig, PricingConfig, PricingTier,
    TechnicalConfig, TechnicalStandard, UseCase, UseCasesConfig, ValueProp,
};
pub use models::{
    CampaignLink, CampaignPerformance, Content, ContentJourneyNode, ContentKind,
    ContentLinkMetadata, ContentMetadata, CreateContentParams, CreateLinkParams, DestinationType,
    IngestionOptions, IngestionReport, LinkClick, LinkPerformance, LinkType, PaperMetadata,
    PaperSection, RecordClickParams, SearchFilters, SearchRequest, SearchResponse, SearchResult,
    Tag, TrackClickParams, UtmParams,
};
pub use navigation::{
    HeaderNavConfig, NavCta, NavItem, NavLink, NavSection, NavigationPageDataProvider,
};
pub use skills_page::{SkillEntry, SkillsPageConfig, SkillsPagePrerenderer};

pub use extenders::{OrgUrlExtender, ReleaseVersionExtender};
pub use jobs::ContentIngestionJob;
pub use repository::{
    ContentRepository, LinkAnalyticsRepository, LinkRepository, SearchRepository,
    UpdateContentParams, UpdateContentParamsBuilder, UpdateContentSeed,
};
pub use services::{
    ContentService, IngestionService, LinkAnalyticsService, LinkGenerationService, LinkService,
    SearchService, ValidationService,
};
