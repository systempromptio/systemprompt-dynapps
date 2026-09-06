# 2026-09-05 — Three audience marketplaces, a role for knowledge workers, and a free knowledge bank

**Status:** decided (Ed, 2026-09-05). Builds on `2026-09-05-groups-projects-roles.md`.

## Decisions

1. **Groups are labelled by content, not by the AD group that feeds them.**
   `europe-devs` is the Commerce Cloud / Storefront Next team (Victor's
   `sfcc-next-cursor` suite) and is fed by AD `Systemprompt-Commerce`;
   `india-devs` is the Salesforce Core + business-analysis team (Jay's
   `sf-dev-agent`, Nishit's `mcp-ba-platform`) and is fed by AD
   `Systemprompt-Core`. `commerce-devs` is retired (migration `043`).

2. **A flat role separates knowledge workers from developers.** `knowledge_worker`
   joins the role set; AD `Systemprompt-Business` projects it, and both dev AD
   groups now project `developer`. The Cowork marketplace is gated on the role
   (`roles: [knowledge_worker]`), not on a group, so an admin can hand it to a
   developer with one manual role grant.

3. **Four marketplaces, two plugins each.** `enterprise-demo` (baseline, every
   `user`), `astound-europe-dev` (`astound-europe-storefront`,
   `astound-europe-delivery`), `astound-india-dev` (`astound-india-sfcore`,
   `astound-india-ba`), `astound-cowork` (`astound-cowork-clients`,
   `astound-cowork-deliverables`). Every audience plugin lists the
   `knowledge-bank` server — Claude Code only sees a server through a plugin's
   `.mcp.json`. A skill is an action with a measured result, and step one of
   every skill grounds itself with `search_knowledge`.

4. **The knowledge bank runs enabled on a free `local` model provider.**
   pgvector stays a hard boot requirement; Bedrock and its key are needed only
   when `model.provider: bedrock`. Ingestion, grants, retrieval and all six
   tools are the live code paths; only embeddings (hashed n-grams) and reranking
   (BM25) are local. The fixture corpus under `services/knowledge-bank/fixtures`
   is indexed through the real `sync_knowledge` tool (`just knowledge-seed`).

5. **Marketplace identity is on the wire and on disk.** Core `next`
   (`ec26f7626`) adds `marketplaces[]` to the signed manifest and the bridge
   writes one Claude Code marketplace directory per granted marketplace,
   replacing the single `org-provisioned` directory. "Received only marketplace
   Y" is therefore an on-disk assertion, not an inference from plugin ids.

6. **Downstream connectors ship disabled until their credentials exist.** The
   existing rule stands: the Salesforce server stays `enabled: false`, and
   because core refuses a plugin that depends on a disabled server, the Cowork
   plugins do not list it yet. The client skills degrade to the knowledge bank.

7. **The Windows client leg is proven server-side only.** The manifest is
   platform-independent and the Windows installer asset is served; no Windows
   runner exists.

## Why

The mechanism landed this morning; what the three audiences needed was content,
a role that distinguishes them, a knowledge bank that works without a paid
model, and a proof that a real client receives exactly its marketplace.

## Consequences

- Proof: `just e2e --grep @REQ-marketplaces` (20 tests: per-principal manifest
  set equality, plugin-file 404, non-console PAT path, Windows leg, a scripted
  `search_knowledge` tool call audited on the mock route) and
  `E2E_CLIENT=host just e2e-marketplaces` (real Claude Code and OpenCode
  enrolments: on-disk marketplace sets, `search_knowledge` over the loopback
  proxy, `mcp_tool_executions` rows).
- The contract baseline gains a `knowledge-worker` column equal to `non-admin`.
- Every signed-in user, the `unassigned` bucket included, reaches the
  knowledge-bank server; what they may read is decided per document.
