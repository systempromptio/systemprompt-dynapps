# Tech debt — gateway wires, token accounting, and the tooling that hid both

Written 2026-09-05, from a session that set out to regression-test one Gemini
tool-call bug and found the bug was a symptom.

Everything below was **measured against a running gateway**, not inferred from
reading code. Where something is unverified it says so. Commands are given so
each finding can be reproduced from a cold start rather than taken on trust.

Status key: **LIVE** = defect is in shipped code right now. **FIXED** = landed
and verified. **IN FLIGHT** = being worked as of writing.

---

## 0. How to reproduce any of this

```bash
# Build and run the current tree (debug locally — release holds the shared
# cargo lock for tens of minutes and blocks every other agent on the clone)
just vendor-core && just migrate && just prepare && just build && just start

# Mint the two credentials the gateway wants. A PAT alone is NOT enough:
# /v1/messages needs a minted session, and an arbitrary x-session-id is
# rejected with "unknown or revoked session" rather than created on demand.
PAT=$(./target/debug/systemprompt admin users api-key issue \
        --user <user-id> --name scratch | grep -o 'sp-live-[^ ]*')
SID=$(curl -sS -X POST localhost:8080/api/public/gateway/sessions \
        -H "Authorization: Bearer $PAT" -H 'content-type: application/json' \
        -d '{}' | jq -r .session_id)

# The full live wire matrix: 3 inbound surfaces x 5 models x buffered/streaming
scripts/e2e-wires.sh "$PAT"
```

---

## 1. Terminal-signal drops — tool calls silently discarded

### The class

A tool call is produced correctly, but **the terminal signal accompanying it is
wrong**, so every conforming client ends the turn and throws the call away. It
is invisible: no error, no log, no failed request. It looks exactly like "the
model chose not to use tools". Any test that asks only *"is there a tool call in
the body?"* passes throughout.

This class has now produced **two independent live bugs on two different wires**,
which is the reason to treat it as a class rather than as two incidents.

### 1a. Gemini reports STOP for a function call — **FIXED** (core `3a45e861b`)

Gemini sets `finishReason: "STOP"` even when the candidate it returns *is* a
`functionCall`. The gateway rendered `finish_reason: "stop"` beside a
fully-formed `tool_calls` array. **All tool use on Gemini through the
OpenAI-compatible surface was dead, MCP included.**

Reproduce the fixed behaviour:

```bash
curl -sS -X POST localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer $PAT" -H "x-session-id: $SID" \
  -H 'content-type: application/json' -d '{
   "model":"gemini-2.5-flash","max_tokens":512,
   "messages":[{"role":"user","content":"Weather in Paris? You must call get_weather."}],
   "tools":[{"type":"function","function":{"name":"get_weather","description":"weather",
     "parameters":{"type":"object","properties":{"city":{"type":"string"}},"required":["city"]}}}]}' \
  | jq '{finish:.choices[0].finish_reason, tool:.choices[0].message.tool_calls[0].function.name}'
# expected now:  {"finish":"tool_calls","tool":"get_weather"}
# pre-fix:       {"finish":"stop",       "tool":"get_weather"}   <- the bug
```

### 1b. `openai_chat` streaming double `MessageStop` — **FIXED** (core `ff0778c74`)

Found *by writing the regression matrix for 1a*, not by a report. The streaming
parser emitted `MessageStop` twice: once from a chunk's real `finish_reason`,
then unconditionally with `EndTurn` on the `[DONE]` sentinel. Providers send
both. The accumulator does `state.final_stop_reason = stop_reason` — assignment,
not merge — so the sentinel's `EndTurn` **overwrote a genuine `tool_calls`
finish on every openai-chat upstream**.

This would have hit all nine `vertex-maas` MaaS models the moment their
credential lands, and would have presented as "the credential works but the
models are broken" — sending the next engineer into the credential path.

### 1c. Systematic audit of all four wires — **IN FLIGHT**

Two of four wires had this bug. **`anthropic` and `openai_responses` have not
been systematically audited**, and neither has the truncation interaction on any
wire. Outstanding checks, per wire, per direction, per mode:

| check | why it matters |
|---|---|
| tool call always yields `CanonicalStopReason::ToolUse` | 1a's shape |
| terminal event cannot be emitted twice or overwritten | 1b's shape |
| terminal event never precedes full tool-block accumulation | drops the call |
| `max_tokens` cutoff mid-tool-call | a `length` finish hiding a partial call |

**Known structural residual:** the Anthropic inbound renders `MessageStop`
through `render_event`, which the tap's `message_stop_rendered` guard does *not*
cover — that guard only gates `render_terminal_event`. With 1b fixed there is no
second `MessageStop` to render today, so this is currently unreachable, but the
surface is unprotected against the next source of a duplicate.

---

## 2. Reasoning tokens are neither counted nor billed — **LIVE**, IN FLIGHT

### Evidence

```bash
curl -sS -X POST localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer $PAT" -H "x-session-id: $SID" \
  -H 'content-type: application/json' -d '{
    "model":"gemini-2.5-flash","max_tokens":200,
    "messages":[{"role":"user","content":"Prove rigorously, step by step, that there are infinitely many primes p such that p+2 is composite. Show all reasoning."}]}' \
  | jq '{finish:.choices[0].finish_reason, usage:.usage}'
```

```json
{"finish":"length","usage":{"prompt_tokens":27,"completion_tokens":6,"total_tokens":33}}
```

`finish_reason: "length"` means the budget was **exhausted** — roughly 200 tokens
were spent. We recorded 6, and `total_tokens` is 27+6, so the arithmetic omits
thinking entirely.

### Root cause

`crates/shared/models/src/wire/canonical/response.rs:20`:

```rust
pub struct CanonicalUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    pub cache_creation_tokens: u32,
    pub total_tokens: u32,
}
```

**There is no field for reasoning tokens.** Every wire drops the provider's count
by construction — this is a canonical-model gap, not a Gemini patch.
`CanonicalUsageUpdate` has the same hole for streaming. Downstream, the
`ai_requests` table -- which is **core-owned** (`crates/domain/ai/schema/`), not
astound's, despite appearing in astound's database because the database is
shared -- has no column either:

```sql
select column_name from information_schema.columns
 where table_name='ai_requests' and column_name like '%token%';
-- cache_creation_tokens, cache_read_tokens, input_tokens,
-- max_tokens, output_tokens, tokens_used
```

### Why it matters commercially

The product is sold on accurate cost attribution. Reasoning spend is currently
invisible **and unbilled**, product-wide. It is worst precisely where it is the
whole point: `qwen.qwen3-next-thinking`, `moonshotai.kimi-k2-thinking`,
`o4-mini`, and Gemini 2.5 with a thinking budget. Those models will appear to
cost almost nothing while consuming most of their budget.

### It also explains a real observed failure

`e2e-opencode.sh` check 6 fails — no MCP tool is invoked. The audit row:

```
input_tokens 9479 | output_tokens 0 | tool_calls (empty) | cost $0.002844
```

The model spent its entire budget thinking and returned an empty candidate. We
billed for input and recorded nothing else. Ruled out: the `format: "uint32"`
warnings OpenCode emits about our MCP schemas are **not** the cause — injecting
that format into a tool definition still returns normal text.

### Scope for a world-class fix

1. `reasoning_tokens: u32` on `CanonicalUsage`, `Option<u32>` on
   `CanonicalUsageUpdate` (the `Option` matters — the struct's own doc comment
   explains that an unreported count and a reported zero must stay
   distinguishable).
2. Capture per provider, buffered **and** streaming:
   - Gemini `usageMetadata.thoughtsTokenCount`
   - OpenAI responses `usage.output_tokens_details.reasoning_tokens`
   - OpenAI chat `usage.completion_tokens_details.reasoning_tokens`
   - Anthropic — bills thinking as ordinary output; set 0 **with a comment
     saying so**, or a later reader will assume it was overlooked.
3. **Settle the inclusion semantics per provider and document them where the
   arithmetic lives.** OpenAI *includes* reasoning in `completion_tokens`;
   Gemini's `thoughtsTokenCount` is *separate from* `candidatesTokenCount`.
   Getting this backwards means double-billing or under-billing. This is the
   single highest-risk detail in the whole change.
4. Price it (output rate unless a provider genuinely differs). Any new field on
   `ModelPricing` must be optional — `#[serde(deny_unknown_fields)]` is in play
   and a required field breaks the boot for every existing `providers.yaml`.
5. `reasoning_tokens` column on `ai_requests`, as a **core** migration under
   `crates/domain/ai/schema/migrations/` (max is 019, so 020) -- astound's
   vacated-slot hazard in §6f does not apply there. Surface it in
   `infra logs request list`, `infra logs audit`, and the analytics rollups;
   astound's analytics repositories need to select the new column too. Note
   that changing those `query_as!` macros invalidates core's checked-in
   `.sqlx/` cache, so `cargo sqlx prepare` must be re-run in core against a
   database with the migration applied, or core will not compile offline.
6. Tests: per-wire parse tests from each provider's real payload shape, plus a
   pricing test proving the tokens reach `cost_dollars`.

---

## 2b. Reasoning-token follow-ups left open, deliberately

The accounting fix (core `5c6040250`, `7895a83e5`) rests on one invariant,
documented at the module head of `wire/canonical/response.rs`: **`reasoning_tokens`
is a breakdown OF `output_tokens`, never an addition to it.** Each adapter
normalises to that -- Gemini's `thoughtsTokenCount` is *additional* upstream so
it is folded in; both OpenAI wires already *include* it so they are copied
through; Anthropic has no field and is set to 0 with a comment saying that is
deliberate. Cost is computed from `output_tokens`, so reasoning bills at the
output rate with no per-provider arithmetic and nothing charged twice. Two tests
pin the two directions: a Gemini payload must bill 2,027 microdollars (under-bill
guard) and an OpenAI payload must bill 106 output tokens rather than 206
(double-bill guard).

Three things that invariant does **not** yet cover:

1. **OpenAI-compatible third parties are assumed, not established.** Cerebras,
   Moonshot and Qwen are treated as following OpenAI's inclusion semantics purely
   because they copy the field name. No evidence either way. If one reports
   reasoning *additionally*, that provider under-bills -- it cannot double-bill,
   since the count is never added, so this is safe to ship. Settling it needs a
   live probe against **Cerebras**, the only one currently routed to. The fix, if
   needed, is a per-provider flag on the catalog entry.

2. **The internal (non-gateway) AI path records no breakdown.** The `INSERT` at
   `repository/ai_requests/mutations.rs:193` leaves `reasoning_tokens` NULL, and
   that is correct rather than a stopgap: the count genuinely does not exist at
   that call site -- `TokenInfo` -> `AiRequestRecord` -> `AiResponse` carry no
   reasoning field -- and the column is nullable so "not recorded" stays
   distinguishable from "the model did no thinking". Writing 0 would assert the
   second and corrupt any later `AVG`/`SUM`. **Nothing is under-billed:** both
   internal cost paths (`ai_service/stream_wrapper.rs`, `ai_service/planning.rs`)
   multiply `output_tokens`, which is already reasoning-inclusive. Only the
   reporting breakdown is missing. Closing it means threading the field through
   `TokenInfo`/`AiRequestRecord`/`AiResponse`/`StreamChunk`, which touches
   `AiResponse`'s serde contract and invalidates the sqlx cache -- a contained
   follow-up, sequenced migration-first.

3. **Core's own migration slots have no tombstone protection.** The mechanism
   added in `86037cbc7` covers only astound's web extension; there are no
   `.tombstone` files under `crates/domain/ai/schema/migrations/`. Core is
   therefore still exposed to the reuse incident that crash-looped production
   (slot 27, `demo_user_uuids` then `usage_loc_columns`, with the per-tenant image
   tag already overwritten so there was no rollback). Slot 020 was confirmed free
   by `git log --all --diff-filter=A` -- by hand, which is the point: reviewer
   diligence is the only guard core has. Extending tombstones to core's migration
   directories is the fix.

---

## 3. `openai_chat` drops reasoning **text** when streaming — **LIVE**

Buffered was fixed in `ff0778c74`: `render_assistant_message` emits
`reasoning_content` and `response.rs` parses it back (with `alias = "reasoning"`
for the providers using the shorter spelling). **The streaming delta path in
`streaming.rs` still ignores `delta.reasoning_content`.**

So a client *streaming* a thinking model gets no reasoning trace and — per §2 —
no reasoning cost either. For `qwen.qwen3-next-thinking` and
`moonshotai.kimi-k2-thinking` that is the entire product, and the proxy this
deployment is replacing renders it today. This is a visible regression for those
users, not a cosmetic gap.

---

## 4. Gemini silently degrades URL images — **LIVE** (warning only)

`wire/gemini/request.rs`, `image_part`: a URL image becomes a plain text part.
It now emits a `tracing::warn!` naming the URL, where previously it was silent.

**Not fixed, deliberately.** `generateContent` has no URL image part — the only
shapes are `inlineData` (base64) or a Files API handle. Rendering it correctly
means fetching the URL or uploading it: network I/O, inside a synchronous wire
codec with no HTTP client, no timeout budget and no error channel. Doing it
there would be worse than the warning.

**Correct fix, for whoever picks this up:** resolve URL images to inline data one
layer up, in the outbound adapter, which *does* have the client — before
`build_body` is called. Budget it as real work with timeouts, a size cap, and a
failure path that surfaces to the caller rather than degrading silently.

---

## 5. The structural test gap that let all of this ship

`crates/tests/integration/api/protocol_matrix.rs` — the file named after this
exact concern — had **19 tests, zero mentions of `gemini`, two of `tool`**, and
asserted only `CanonicalStopReason::EndTurn`, never `ToolUse`.

More fundamentally: of the **12 inbound × outbound combinations, 11 had no
joined coverage at all**. Every test was inbound-only (parse a body, render a
hand-built response) or outbound-only (build from a hand-built request, parse a
canned body). Only `anthropic_messages → anthropic` was exercised end to end.

**That gap is why the bug shipped**, and why it produced two bugs rather than one.

### Now in place

- **Offline matrix** (core, `gateway_matrix*.rs`): 24 cells, 3 inbound × 4
  outbound × buffered/streaming, driven against `wiremock`, **no credentials
  required** so it runs in CI. Every cell asserts both that a tool call survives
  *and* that the terminal reason declares it.
- **Live matrix** (`scripts/e2e-wires.sh`): 30 cells against real upstreams,
  same two assertions, plus audit-trail verification. **Currently 30/30 green.**

### Caveats — do not over-read the green

- **Two of the twelve cells prove less than the rest.** `anthropic-in →
  anthropic-out` runs the raw byte-passthrough lane in both directions, so it
  asserts passthrough, not translation. `openai_chat-in → openai_chat-out` uses
  the raw lane on the request only; its response assertion is real. So it is 10
  translated cells plus 2 passthrough ones, not 12 equivalent cells.
- **Truncation mid-tool-call is not covered by either matrix.**
- The offline matrix was authored without being compiled or run in the core
  checkout (core CI is that repo's validation surface). It must be run from
  astound, which compiles the patched core, before it is trusted.

---

## 6. Tooling that actively hides failures

These cost more time than the bugs. Each is a small fix with a large payoff.

### 6a. A failed `just prepare` deletes the `.sqlx` cache — **LIVE**

A `prepare` that errors part-way leaves the cache **worse than it found it**.
Observed: 55 entries deleted before the error. Recovery is `git checkout -- .sqlx`,
which is not obvious mid-incident.

*Fix:* write to a temp dir and swap on success only.

### 6b. `migrate-repair --apply` corrupts rows silently — **LIVE**

It will write one migration's checksum onto a **differently-named** row, with no
name check and no warning, while reporting `0 migration(s) re-applied`. Observed:
it stamped a new file's checksum onto the `knowledge_bank` row, producing a row
describing neither migration truthfully and **permanently silencing that row's
drift detector**.

The `--apply` flag implies execution; it reconciled bookkeeping without running
any SQL. So it asserts the DB matches the file without ever checking.

*Fix:* refuse when the recorded `name` differs from the file's; say plainly
whether SQL ran; make `--reconcile-only` the only path that skips execution.

### 6c. Assertions that skip themselves — **FIXED** in `scripts/e2e-wires.sh`

The harness gated its audit checks on `command -v systemprompt`, which is not on
PATH in this repo, so it printed "skipping" and **still declared success**. Its
tool-call check counted `ai_request_tool_calls` all-time, so 335 historical rows
would have carried it green even if the run recorded nothing.

*General rule, worth enforcing repo-wide:* a skipped assertion is
indistinguishable from a passed one. Skips must fail, or be counted and printed
in the summary. This is the same failure mode as the bug the harness exists to
catch.

### 6d. `pgrep -f` watchers that can never exit

`until ! pgrep -f "sqlx prepare"; do sleep 5; done` — the watcher's own `bash -c`
command line *contains* the needle, so it matches itself and loops forever. Six
were found spinning after their target had finished; I then made the identical
mistake with `just build --release` and got a watcher reporting a failed build as
still running.

*Fix:* use a sentinel file, or `pgrep -f <pat> | grep -v $$`.

### 6e. The migrations directory is compiled source

`extension_migrations!()` bakes absolute paths into `include_str!` at
macro-expansion time. Renaming a file under `schema/migrations/` during a build
fails the compile pointing at a path that no longer exists. Worse,
`extensions/web/schema/05_plugin_usage.sql` is `include_str!`'d at
`extensions/web/src/schemas.rs:9` — a **content** edit read mid-write embeds a
truncated file **rather than failing**.

*Rule:* "a build is running" covers the whole tree, not just `.rs`.

### 6f. Vacated migration slots — **FIXED** (core `86037cbc7`)

A numbering gap may be a *spent* slot. Slot 034 looked free; `034_knowledge_bank.sql`
had shipped in v0.42.0–v0.46.0 and every deployed database had it applied. A new
`034_*.sql` would have been **silently skipped everywhere except a fresh local
DB** — the view would never have existed in production and both new admin pages
would have broken at runtime.

It passed locally only because the view had been created *by hand*. Now enforced
twice: `build.rs` fails the build on a refilled slot, and
`scripts/check-migration-numbers.sh` catches it in seconds via `lint-gates`.

---

### 6g. `crates/tests` is a separate workspace — the main one does not build it

A change to a domain crate can compile cleanly in the main workspace while
leaving the **test** workspace red, because the main workspace does not build
it. This happened **three times in one session**, each time taking down most of
`crates/tests` for every session sharing the clone, and each time the author had
run `cargo check` and seen green.

`cargo check` in the main workspace is not evidence. After changing a shared
type, check the test workspace explicitly:

```bash
cargo check --manifest-path crates/tests/Cargo.toml --workspace   # core
cargo check --manifest-path tests/Cargo.toml --workspace          # astound
```

The specific shape that recurred: a column added to a struct, the obvious
queries updated, and every *other* projection of that struct missed. The check
that finds them all in seconds is to grep the projection, not the field —
`grep -rn 'cache_read_tokens, cache_creation_tokens'` located all four sites
where `grep reasoning_tokens` located none of them.

### 6h. A migration and its queries cannot land atomically

In a shared clone with a pinned compile-time `DATABASE_URL` (core keeps one in
`.cargo/config.toml`), the instant a query naming a new column is on disk,
**every session's build breaks** until that database has the column. There is no
ordering of a single commit that avoids this.

The DB step has to be sequenced deliberately and separately: apply the migration
to every live database first, then commit the queries. Note "every live
database" is not obvious either — on this host it meant six, including core's
pinned `local_19c2d6bdf6c` and the partitioned `systemprompt_test_w2/w3/w4`.

### 6i. Untracked files under `crates/tests` are work, not debris

**This is the standing hazard and it cost a full day.** The `.gitignore` makes
untracked files under `crates/tests` look like build residue. They are often the
work in progress of another session — around 30 test modules and 800+ tests in
one incident, unrecoverable because untracked files never enter git: no stash,
no reflog, no dangling objects.

The vendor mirror is **not** a backup either. `just vendor-core` uses
`rsync -a --delete`, so re-vendoring after a deletion faithfully reproduces it.

Rules that follow, for any checkout more than one session is writing to:

- Never run `git clean`, `git checkout .`, `git reset --hard` or `git stash`
  there. Use explicit paths: `git checkout -- 'crates/*/*/.sqlx'`, never
  `git checkout -- crates/`.
- Commit early, even half-finished. A commit on a branch is recoverable; an
  untracked file is not.
- When a guard refuses, reconcile *every* number before overriding it. A prune
  guard reporting 41 removed queries alongside 324 deleted files is describing
  two different things, and the discrepancy — not the reported figure — is the
  signal.

## 7. Priority

| # | Item | Status | Why this rank |
|---|---|---|---|
| 1 | §1c audit of `anthropic` + `openai_responses` terminal signals | IN FLIGHT | Two of four wires had a live silent-drop; the other two are unaudited |
| 2 | §2 reasoning-token accounting | IN FLIGHT | Unbilled revenue and a false cost story in a product sold on cost attribution |
| 3 | §3 streaming `reasoning_content` | LIVE | Visible regression vs the proxy being replaced, for the models whose purpose is reasoning |
| 4 | §6b `migrate-repair` | LIVE | Silently corrupts the ledger and disables the detector meant to catch it |
| 5 | §5 truncation-mid-tool-call coverage | Not started | The one terminal-signal shape neither matrix tests |
| 6 | §6a `prepare` cache deletion | LIVE | Cheap fix, recurring cost |
| 7 | §4 Gemini URL images | LIVE (warned) | Correctly deferred; needs a layer change, not a patch |

---

## 8. What is genuinely proven today

- 30/30 live cells green across `anthropic`, `gemini`, `openai-chat` (OpenAI and
  Cerebras) and `openai-responses`, on all three inbound surfaces, buffered and
  streaming, each asserting both halves.
- A **negative control** confirms the harness bites: fed the pre-fix shape
  (`finish_reason:"stop"` beside a populated `tool_calls`) it fails and names the
  bug rather than passing.
- Audit spine complete: five providers costed, upstream model names resolved, 61
  `get_weather` rows in `ai_request_tool_calls`.
- `/v1/models` = 21, with all nine uncredentialed `vertex-maas` models correctly
  demoted rather than advertised-and-502ing.
- Vendored-core clippy clean across `systemprompt-api`, `-models`, `-loader`,
  `-cli`.

## 9. What is NOT proven

- The 24 offline matrix cells have **never been executed** — written in the core
  checkout, which does not compile or test by policy. Run them from astound.
- `anthropic` and `openai_responses` have not been audited for §1.
- No reasoning-token work is verified; it is mid-flight.
- `e2e-opencode.sh` check 6 is **red**, and correctly so — it is failing on §2,
  not on the wire bug. It has deliberately not been loosened to pass.
