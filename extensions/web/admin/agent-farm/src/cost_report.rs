//! The Cost page's totals: farm tasks joined to systemprompt's per-session
//! costs, by agent, person and tenant.

use crate::client::is_all_tenants;
use crate::models::TaskOut;
use crate::types::SessionTotalRow;
use crate::usage::{count_label, format_usd, gateway_session_id};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Default, Clone, Copy)]
pub struct Bucket {
    tasks: u64,
    unlinked: u64,
    calls: i64,
    cost: i64,
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
}

impl Bucket {
    const fn add(&mut self, total: Option<&SessionTotalRow>, linked: bool) {
        self.tasks += 1;
        if !linked {
            self.unlinked += 1;
        }
        if let Some(t) = total {
            self.calls += t.calls;
            self.cost += t.cost_microdollars;
            self.input_tokens += t.input_tokens;
            self.output_tokens += t.output_tokens;
            self.cache_read_tokens += t.cache_read_tokens;
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CostRow {
    pub key: String,
    pub tasks: u64,
    pub calls: i64,
    pub cost: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub unlinked: u64,
    #[serde(skip)]
    sort_cost: i64,
}

#[derive(Debug, Serialize)]
pub struct CostReport {
    pub total_cost: String,
    pub tasks_label: String,
    pub calls_label: String,
    pub unlinked_label: Option<String>,
    pub empty_label: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub by_agent: Vec<CostRow>,
    pub by_tenant: Vec<CostRow>,
    pub by_person: Vec<CostRow>,
}

pub fn rows(buckets: BTreeMap<String, Bucket>) -> Vec<CostRow> {
    let mut out: Vec<CostRow> = buckets
        .into_iter()
        .map(|(key, b)| CostRow {
            key,
            tasks: b.tasks,
            calls: b.calls,
            cost: format_usd(b.cost),
            input_tokens: b.input_tokens,
            output_tokens: b.output_tokens,
            cache_read_tokens: b.cache_read_tokens,
            unlinked: b.unlinked,
            sort_cost: b.cost,
        })
        .collect();
    out.sort_by(|a, b| {
        b.sort_cost
            .cmp(&a.sort_cost)
            .then(b.tasks.cmp(&a.tasks))
            .then(a.key.cmp(&b.key))
    });
    out
}

pub fn sessions(tasks: &[(String, TaskOut)]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    tasks
        .iter()
        .filter_map(|(_, t)| task_session(t))
        .filter(|s| seen.insert(s.clone()))
        .collect()
}

pub fn task_session(task: &TaskOut) -> Option<String> {
    gateway_session_id(
        task.usage
            .as_ref()
            .and_then(|u| u.gateway_session_id.as_deref()),
    )
}

pub fn aggregate(tasks: &[(String, TaskOut)], totals: &[SessionTotalRow]) -> CostReport {
    let by_session: HashMap<&str, &SessionTotalRow> = totals
        .iter()
        .map(|t| (t.gateway_session.as_str(), t))
        .collect();
    let mut agents: BTreeMap<String, Bucket> = BTreeMap::new();
    let mut tenants: BTreeMap<String, Bucket> = BTreeMap::new();
    let mut people: BTreeMap<String, Bucket> = BTreeMap::new();
    let mut all = Bucket::default();
    let mut empty = 0i64;
    for (read_from, task) in tasks {
        let agent = task
            .agent
            .clone()
            .filter(|a| !a.is_empty())
            .unwrap_or_else(|| "free prompt".to_owned());
        let tenant = task
            .tenant
            .clone()
            .filter(|t| !t.is_empty())
            .or_else(|| Some(read_from.clone()).filter(|t| !is_all_tenants(t)))
            .unwrap_or_else(|| "unknown".to_owned());
        let person = task
            .owner
            .clone()
            .filter(|o| !o.is_empty())
            .unwrap_or_else(|| "unknown".to_owned());
        let session = task_session(task);
        let total = session.as_deref().and_then(|s| by_session.get(s).copied());
        let linked = session.is_some();
        if linked && total.is_none() {
            empty += 1;
        }
        agents.entry(agent).or_default().add(total, linked);
        tenants.entry(tenant).or_default().add(total, linked);
        people.entry(person).or_default().add(total, linked);
        all.add(total, linked);
    }
    CostReport {
        total_cost: format_usd(all.cost),
        tasks_label: count_label(tasks.len() as i64, "task", "tasks"),
        calls_label: count_label(all.calls, "model call", "model calls"),
        unlinked_label: (all.unlinked > 0).then(|| format!("{} without SP link", all.unlinked)),
        empty_label: (empty > 0).then(|| format!("{empty} linked with no calls in systemprompt")),
        input_tokens: all.input_tokens,
        output_tokens: all.output_tokens,
        cache_read_tokens: all.cache_read_tokens,
        by_agent: rows(agents),
        by_tenant: rows(tenants),
        by_person: rows(people),
    }
}
