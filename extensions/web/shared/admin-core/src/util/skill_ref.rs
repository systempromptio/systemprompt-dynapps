//! The one form a skill has once it leaves `services/`.
//!
//! A skill is declared as `services/skills/<snake_id>/`, and every host
//! presents it dashed: Claude Code lists `/<plugin>:<dashed-id>`, the bridge
//! writes `skills/<dashed-id>/SKILL.md`, and the hooks report the dashed
//! name back. Analytics therefore key on the dashed `plugin:skill` — the
//! `analysis_skill_events` view, the fact pipeline and the marketplace hash
//! all agree on it — and anything that starts from a services id must go
//! through here rather than compare the two spellings.

#[must_use]
pub fn skill_ref(plugin: &str, skill: &str) -> String {
    format!("{plugin}:{}", host_skill_name(skill))
}

#[must_use]
pub fn host_skill_name(skill: &str) -> String {
    skill.replace('_', "-")
}
