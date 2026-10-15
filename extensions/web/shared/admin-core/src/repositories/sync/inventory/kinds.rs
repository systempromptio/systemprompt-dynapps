//! The `CONFIG_KINDS` table itself. Order is display order: projected kinds
//! first, in plane order, then everything served from code, grouped the way
//! the `services/` tree is — the configuration page prints them as listed.

use super::{ConfigKind, dir, file, paged, projected};

pub const CONFIG_KINDS: &[ConfigKind] = &[
    projected(
        file(
            "access_control",
            "Access control",
            "who reaches which marketplace, plugin, skill, server and route",
            "access-control/rules.yaml",
        ),
        "access_control",
        "/admin/access-control",
    ),
    projected(
        file(
            "groups",
            "Groups and projects",
            "the containers people land in and the directory groups that place them",
            "web/config/groups.yaml",
        ),
        "groups",
        "/admin/groups",
    ),
    projected(
        file(
            "gateway_policies",
            "Gateway policies",
            "quota windows, safety scanners and block lists on every inference request",
            "gateway/policies.yaml",
        ),
        "gateway_policies",
        "/admin/gateway/policies",
    ),
    projected(
        file(
            "gateway_routes",
            "Gateway routes",
            "which model names dispatch to which provider",
            "ai/gateway.yaml",
        ),
        "gateway_routes",
        "/admin/gateway",
    ),
    projected(
        file(
            "governance",
            "Governance chain",
            "the four request-time stages every tool call passes through",
            "governance/config.yaml",
        ),
        "governance",
        "/admin/governance",
    ),
    file(
        "config",
        "Root aggregator",
        "the includes list and instance settings every flat file hangs off",
        "config/config.yaml",
    ),
    paged(
        dir(
            "marketplaces",
            "Marketplaces",
            "what a signed-in user is offered; a content hash is its version",
            "marketplaces",
        ),
        "/admin/marketplaces",
    ),
    paged(
        dir(
            "plugins",
            "Plugins",
            "binding descriptors that group skills, servers and hooks by reference",
            "plugins",
        ),
        "/admin/plugins",
    ),
    paged(
        dir(
            "skills",
            "Skills",
            "the instruction bodies the bridge ships to Claude Code",
            "skills",
        ),
        "/admin/skills",
    ),
    paged(
        dir(
            "mcp",
            "MCP servers",
            "server definitions the instance supervises and proxies",
            "mcp",
        ),
        "/admin/mcp",
    ),
    dir(
        "artifacts",
        "Artifacts",
        "dashboards rendered from a skill's output",
        "artifacts",
    ),
    file(
        "providers",
        "Provider catalog",
        "every provider and model the gateway can reach, with pricing",
        "ai/providers.yaml",
    ),
    file(
        "ai_config",
        "AI defaults",
        "the default provider and the inference settings behind it",
        "ai/config.yaml",
    ),
    file(
        "scheduler",
        "Scheduler",
        "the jobs the instance runs on a timer",
        "scheduler/config.yaml",
    ),
    dir(
        "external_agents",
        "External agents",
        "the clients the bridge recognises",
        "external_agents",
    ),
    dir(
        "content",
        "Content",
        "documentation and other sources the publish pipeline ingests",
        "content",
    ),
    file(
        "blog",
        "Blog ingestion",
        "where posts come from and how they are published",
        "config/blog.yaml",
    ),
    file(
        "salesforce_orgs",
        "Salesforce orgs",
        "the org registry behind the per-org MCP servers",
        "config/salesforce-orgs.yaml",
    ),
    file(
        "agent_farms",
        "Agent farms",
        "the Git-sourced agent farms this instance syncs and runs",
        "web/config/agent-farms.yaml",
    ),
    file(
        "web_config",
        "Web config",
        "the public site's settings",
        "web/config.yaml",
    ),
    file(
        "web_metadata",
        "Site metadata",
        "titles, descriptions and social cards",
        "web/metadata.yaml",
    ),
    file(
        "entra_oidc",
        "Entra ID sign-in",
        "the OpenID Connect app registration the console signs in through",
        "web/config/entra-oidc.yaml",
    ),
    file(
        "security",
        "Security policy",
        "session, token and origin rules",
        "web/config/security.yaml",
    ),
    file(
        "homepage",
        "Homepage",
        "the public landing page's sections",
        "web/config/homepage.yaml",
    ),
    file(
        "navigation",
        "Navigation",
        "the public site's menus",
        "web/config/navigation.yaml",
    ),
    file(
        "theme",
        "Theme",
        "brand colours and type",
        "web/config/theme.yaml",
    ),
    dir(
        "web_templates",
        "Web templates",
        "the public site's page shells",
        "web/templates",
    ),
];
