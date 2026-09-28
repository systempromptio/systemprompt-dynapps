//! View-model assembly for the catalog pages.
//!
//! Turns a loaded [`Catalog`] plus per-entity assignment counts into the
//! `Serialize` structs the templates consume. Loading lives in [`super::data`].

use std::collections::HashMap;

use systemprompt::identifiers::{PluginId, SkillId};

use crate::types::{ConfiguredHook, ENTITY_PLUGIN, ENTITY_SKILL, PluginDetail, SkillCatalogEntry};

use super::data::Catalog;
use super::view::{
    HookRef, LinkedEntity, PluginDetailData, PluginListRow, SkillDetailData, SkillListRow,
    access_url, mcp_url, plugin_url, skill_url,
};
use super::visibility::{VisibilityView, carriers_of_plugin, carriers_of_skill, visibility_for};
use crate::handlers::ssr::analysis_urls::{ANALYSIS_SKILLS_URL, analysis_skill_url};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::marketplace::manifests::MarketplaceConfigSummary;
use crate::types::access_control::AccessControlRule;

pub(super) struct VisibilityInput<'a> {
    pub(super) manifests: &'a [MarketplaceConfigSummary],
    pub(super) rules: &'a [AccessControlRule],
}

impl VisibilityInput<'_> {
    fn plugin(&self, plugin_id: &PluginId) -> VisibilityView {
        let carriers = carriers_of_plugin(self.manifests, plugin_id);
        visibility_for(self.rules, ENTITY_PLUGIN, plugin_id.as_str(), &carriers)
    }

    fn skill(&self, skill_id: &SkillId, plugin_ids: &[String]) -> VisibilityView {
        let carriers = carriers_of_skill(self.manifests, plugin_ids);
        visibility_for(self.rules, ENTITY_SKILL, skill_id.as_str(), &carriers)
    }
}

fn trail(listing: &'static str, href: &'static str, current: &str) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link(listing, href),
        BreadcrumbView::current(current.to_owned()),
    ]
}

pub(super) fn plugin_rows(
    catalog: Catalog,
    counts: &HashMap<String, i64>,
    visibility: &VisibilityInput<'_>,
) -> Vec<PluginListRow> {
    catalog
        .plugins
        .into_iter()
        .map(|p| PluginListRow {
            visibility: visibility.plugin(&p.id),
            detail_url: plugin_url(p.id.as_str()),
            access_url: access_url(&plugin_url(p.id.as_str())),
            skills_count: p.skills.len(),
            mcp_count: p.mcp_servers.len(),
            agents_count: p.agents.len(),
            assignment_count: counts.get(p.id.as_str()).copied().unwrap_or(0),
            id: p.id,
            name: p.name,
            description: p.description,
            category: p.category,
            version: p.version,
            enabled: p.enabled,
            source_path: p.source_path,
            source: String::new(),
            source_tone: "",
        })
        .collect()
}

fn name_or_id(names: &HashMap<&str, &str>, id: &str) -> String {
    names
        .get(id)
        .map_or_else(|| id.to_owned(), |n| (*n).to_owned())
}

fn plugin_skills(skills: &[SkillCatalogEntry], plugin: &PluginDetail) -> Vec<LinkedEntity> {
    let names: HashMap<&str, &str> = skills
        .iter()
        .map(|s| (s.id.as_str(), s.name.as_str()))
        .collect();
    plugin
        .skills
        .iter()
        .map(|s| LinkedEntity {
            name: name_or_id(&names, s.as_str()),
            url: skill_url(s.as_str()),
            id: s.as_str().to_owned(),
        })
        .collect()
}

fn plugin_agents(
    agent_names: &HashMap<String, String>,
    plugin: &PluginDetail,
) -> Vec<LinkedEntity> {
    plugin
        .agents
        .iter()
        .map(|a| LinkedEntity {
            name: agent_names
                .get(a.as_str())
                .map_or_else(|| a.as_str().to_owned(), String::clone),
            url: String::new(),
            id: a.as_str().to_owned(),
        })
        .collect()
}

fn plugin_mcp_servers(plugin: &PluginDetail) -> Vec<LinkedEntity> {
    plugin
        .mcp_servers
        .iter()
        .map(|m| LinkedEntity {
            name: m.as_str().to_owned(),
            url: mcp_url(m.as_str()),
            id: m.as_str().to_owned(),
        })
        .collect()
}

pub(super) fn plugin_detail(
    catalog: Catalog,
    plugin_id: &PluginId,
    assignment_count: i64,
) -> Option<PluginDetailData> {
    let Catalog {
        plugins,
        skills: catalog_skills,
        agent_names,
        mut hooks_by_plugin,
        ..
    } = catalog;
    let plugin = plugins.into_iter().find(|p| &p.id == plugin_id)?;
    let skills = plugin_skills(&catalog_skills, &plugin);
    let mcp_servers = plugin_mcp_servers(&plugin);
    let agents = plugin_agents(&agent_names, &plugin);
    let hooks: Vec<HookRef> = hooks_by_plugin
        .remove(plugin.id.as_str())
        .map(|hs| hs.into_iter().map(hook_ref).collect())
        .unwrap_or_default();

    Some(PluginDetailData {
        breadcrumbs: trail("Plugins", "/admin/plugins", &plugin.name),
        page: "plugin-detail",
        title: plugin.name.clone(),
        access_url: access_url(&plugin_url(plugin.id.as_str())),
        assignment_count,
        skills_count: skills.len(),
        mcp_count: mcp_servers.len(),
        agents_count: agents.len(),
        hooks_count: hooks.len(),
        skills,
        mcp_servers,
        agents,
        hooks,
        access: None,
        id: plugin.id,
        name: plugin.name,
        description: plugin.description,
        version: plugin.version,
        category: plugin.category,
        enabled: plugin.enabled,
        author_name: plugin.author_name,
        keywords: plugin.keywords,
        roles: plugin.roles,
        source_path: plugin.source_path,
    })
}

fn hook_ref(h: ConfiguredHook) -> HookRef {
    HookRef {
        id: h.id,
        event: h.event,
        matcher: h.matcher,
        command: h.command,
        is_async: h.is_async,
    }
}

pub(super) fn skill_rows(
    catalog: &Catalog,
    counts: &HashMap<String, i64>,
    visibility: &VisibilityInput<'_>,
) -> Vec<SkillListRow> {
    catalog
        .skills
        .iter()
        .map(|s| {
            let id = s.id.as_str().to_owned();
            let plugin_ids: Vec<String> = catalog
                .plugins_by_skill
                .get(&id)
                .map(|links| links.iter().map(|l| l.id.clone()).collect())
                .unwrap_or_default();
            SkillListRow {
                visibility: visibility.skill(&s.id, &plugin_ids),
                detail_url: skill_url(&id),
                access_url: access_url(&skill_url(&id)),
                assignment_count: counts.get(&id).copied().unwrap_or(0),
                plugin_count: catalog.plugins_by_skill.get(&id).map_or(0, Vec::len),
                id,
                name: s.name.clone(),
                description: s.description.clone(),
                enabled: s.enabled,
                source_path: s.source_path.clone(),
                source: String::new(),
                source_tone: "",
            }
        })
        .collect()
}

fn skill_activity_url(included_by: &[LinkedEntity], skill: &SkillId) -> String {
    included_by.first().map_or_else(
        || ANALYSIS_SKILLS_URL.to_owned(),
        |plugin| {
            analysis_skill_url(&format!(
                "{}:{}",
                plugin.id,
                skill.as_str().replace('_', "-")
            ))
        },
    )
}

pub(super) fn skill_detail(
    catalog: Catalog,
    skill: &SkillId,
    assignment_count: i64,
) -> Option<SkillDetailData> {
    let Catalog {
        skills,
        mut plugins_by_skill,
        ..
    } = catalog;
    let id = skill.as_str();
    let entry = skills.into_iter().find(|s| s.id == *skill)?;
    let included_by = plugins_by_skill.remove(id).unwrap_or_default();
    Some(SkillDetailData {
        breadcrumbs: trail("Skills", "/admin/skills", &entry.name),
        activity_url: skill_activity_url(&included_by, skill),
        page: "skill-detail",
        title: entry.name.clone(),
        access_url: access_url(&skill_url(id)),
        assignment_count,
        included_by_count: included_by.len(),
        included_by,
        access: None,
        id: id.to_owned(),
        name: entry.name,
        description: entry.description,
        enabled: entry.enabled,
        source_path: entry.source_path,
    })
}
