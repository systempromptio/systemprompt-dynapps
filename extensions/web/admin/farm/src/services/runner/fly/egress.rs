//! The runner machine's outbound allowlist, as the API states it and the
//! worker installs it.
//!
//! A runner may reach loopback, Fly's private 6PN network (`fdaa::/16`,
//! which carries the API, the gateway and Fly DNS at `fdaa::3`), the hosts
//! of every URL the run itself names (worker API, gateway, governance hooks,
//! MCP servers) and the `fly.egress_allow` entries of the farms file.
//! Everything else is rejected. [`EgressPolicy`] travels in the
//! [`WorkerSpec`](super::protocol::WorkerSpec); the worker resolves its host
//! names, builds the nftables ruleset with [`nft_ruleset`] and refuses to run
//! the agent if that ruleset cannot be loaded.

use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use systemprompt::agent_runs::{McpServerSpec, RunRequest};

pub const NFT_TABLE: &str = "farm_egress";
pub const FLY_PRIVATE_NET: &str = "fdaa::/16";
pub const FLY_DNS: &str = "fdaa::3";

/// What a runner may reach: host names, IP addresses or CIDR blocks.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EgressPolicy {
    pub allow: Vec<String>,
}

impl EgressPolicy {
    #[must_use]
    pub fn for_run(configured: &[String], request: &RunRequest, urls: &[&str]) -> Self {
        let mcp = request.mcp_servers.values().filter_map(|spec| match spec {
            McpServerSpec::Http { url, .. } => Some(url.as_str()),
            McpServerSpec::Stdio { .. } => None,
        });
        let mut allow: Vec<String> = configured.iter().map(|e| e.trim().to_owned()).collect();
        allow.extend(urls.iter().copied().chain(mcp).filter_map(url_host));
        allow.retain(|entry| !entry.is_empty());
        allow.sort();
        allow.dedup();
        Self { allow }
    }

    #[must_use]
    pub fn label(&self) -> String {
        let mut parts = vec!["lo".to_owned(), FLY_PRIVATE_NET.to_owned()];
        parts.extend(self.allow.iter().cloned());
        format!("allow [{}]; reject all else", parts.join(", "))
    }
}

#[must_use]
pub fn url_host(url: &str) -> Option<String> {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let authority = rest.split(['/', '?', '#']).next()?;
    let authority = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let host = if let Some(v6) = authority.strip_prefix('[') {
        v6.split(']').next()?
    } else {
        authority.split(':').next()?
    };
    (!host.is_empty()).then(|| host.to_owned())
}

/// An address block the ruleset accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Net {
    pub addr: IpAddr,
    pub prefix: u8,
}

impl Net {
    #[must_use]
    pub const fn host(addr: IpAddr) -> Self {
        let prefix = if addr.is_ipv4() { 32 } else { 128 };
        Self { addr, prefix }
    }

    #[must_use]
    pub fn parse(entry: &str) -> Option<Self> {
        let (addr, prefix) = match entry.trim().split_once('/') {
            Some((addr, prefix)) => (addr, Some(prefix.parse::<u8>().ok()?)),
            None => (entry.trim(), None),
        };
        let addr: IpAddr = addr.parse().ok()?;
        let max = if addr.is_ipv4() { 32 } else { 128 };
        let prefix = prefix.unwrap_or(max);
        (prefix <= max).then_some(Self { addr, prefix })
    }

    fn rule(self) -> String {
        let family = if self.addr.is_ipv4() { "ip" } else { "ip6" };
        format!("{family} daddr {}/{} accept", self.addr, self.prefix)
    }
}

#[must_use]
pub fn nft_ruleset(nets: &[Net]) -> String {
    let mut rules = vec![
        "oif \"lo\" accept".to_owned(),
        // Why: Linux sends IPv6 neighbour discovery as output multicast; dropping it
        // empties the neighbour cache after ~30s and cuts every 6PN connection.
        "icmpv6 type { nd-neighbor-solicit, nd-neighbor-advert, nd-router-solicit } accept"
            .to_owned(),
        "ct state established,related accept".to_owned(),
        format!("ip6 daddr {FLY_PRIVATE_NET} accept"),
        format!("ip6 daddr {FLY_DNS} accept"),
    ];
    rules.extend(nets.iter().map(|net| net.rule()));
    rules.push("meta l4proto tcp reject with tcp reset".to_owned());
    rules.push("reject".to_owned());
    let body: String = rules
        .iter()
        .flat_map(|r| ["    ", r.as_str(), "\n"])
        .collect();
    format!(
        "table inet {NFT_TABLE}\ndelete table inet {NFT_TABLE}\ntable inet {NFT_TABLE} {{\n  \
         chain output {{\n    type filter hook output priority 0; policy drop;\n{body}  }}\n}}\n"
    )
}
