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

### 1c. Systematic audit of all four wires — **FIXED** (core `c161d5d81`, `d316add7c`, `479b22c1e`, `59e68ea5c`)

Two of four wires had this bug. **`anthropic` and `openai_responses` have not
been systematically audited**, and neither has the truncation interaction on any
wire. The 24 offline matrix cells that would cover this have also never been
executed (§9). Workstream A4 ran the 24 cells (they were green on first execution) and
added 31 more: `anthropic` and `openai_responses` terminal signals crossed with
all three inbounds, buffered and streaming, plus truncation mid-tool-call and a
Vertex-`MaaS`-style explicit-`null` fixture. Two live defects fell out and are
fixed: the Anthropic-in → Anthropic-out passthrough lane relayed `end_turn`
beside a `tool_use` block uncorrected (`d316add7c` rewrites only the
`stop_reason` token, everything else stays byte-identical), and the buffered
Responses renderer hardcoded `status: "completed"` on a truncated turn and
emitted a `stop_reason` key the Responses contract does not define
(`479b22c1e`, `59e68ea5c`). The checks, per wire, per direction, per mode:

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

## 2. Reasoning tokens — **CORRECTED 2026-09-05**, not fixed

The original entry here claimed reasoning tokens were "neither counted nor
billed" product-wide. That was written before the accounting work in §2b landed
and before anyone measured a live row. It was too broad, and it is corrected
rather than fixed, because half of what it described was never a defect.

### What the original entry got right

`CanonicalUsage` genuinely had no reasoning field, so every wire dropped the
provider's count by construction. That is the gap §2b closed: the field exists,
each adapter normalises to the "breakdown of `output_tokens`" invariant, and
`ai_requests` carries a `reasoning_tokens` column.

### What is true now, measured in `ai_requests`

The gemini wire **does** record reasoning tokens. A `vertex gemini-2.5-pro` row
from the live matrix:

| input | output | reasoning |
|---|---|---|
| 3 | 497 | 491 |

The openai-chat wire records **0** for `moonshotai.kimi-k2-thinking`,
`zai.glm-5` and `qwen.qwen3-next-thinking`. That is not a parser bug. Vertex
MaaS sends no `completion_tokens_details` object at all on those models, so
there is no count to parse — the field the adapter reads is absent from the
upstream payload, and a reported zero is the honest rendering of an unreported
count on a wire whose canonical field is not optional.

### Consequence

Reasoning spend on Vertex MaaS thinking models is invisible **because the
upstream does not report it**, and no change on our side recovers it. Those
tokens are still billed: they are inside `completion_tokens`, which is what the
cost arithmetic uses, so the money is right even though the breakdown is not.
The remaining exposure is a reporting one — a customer asking "how much of this
was thinking?" gets no answer for those three models.

*If Google starts sending `completion_tokens_details` on the MaaS surface, the
existing parser picks it up with no code change.* Nothing is owed here until
then. Re-probe before promising a customer the breakdown.

---

## 2b. Reasoning-token follow-ups — **CLOSED 2026-09-05**

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

All three follow-ups below were closed on 2026-09-05 (core `067e49102`,
`f427ae2ae`, `d05765da8`, `019b0fc9e`). Kept here because the *reasoning* for
each answer is worth more than the fact that it is done.

1. **OpenAI-compatible third parties: settled by observation, then made
   self-enforcing.** Cerebras, Moonshot and Qwen were assumed to follow OpenAI's
   inclusion semantics purely because they copy the field name. A live matrix run
   (`scripts/e2e-wires.sh`, 30/30 cells) shows Cerebras `gpt-oss-120b` reporting
   `146+128c/60(36r)` -- 36 reasoning tokens *inside* 60 output tokens, so it does
   follow the contract, and it also returns cache-read tokens we were not
   crediting before.

   The assumption is no longer load-bearing either way.
   `CanonicalUsage::normalise_reasoning(provider)` now enforces the invariant at
   runtime for every provider, probed or not: a breakdown cannot exceed its
   parent, so `reasoning > output` means the provider reported additionally, and
   a wire `total_tokens` equal to `input + output + reasoning` is the same signal
   caught earlier. Either one folds the count into `output_tokens` and emits a
   `tracing::warn!` naming the provider. It runs at the two points usage leaves
   wire parsing knowing which provider produced it: the gateway audit
   (`audit/complete.rs`, covering buffered and streamed alike) and the internal
   `canonical_bridge/response.rs`. No per-provider flag was needed.

   **One trap found while building it, worth remembering.** The total-based half
   nearly shipped wrong. `CanonicalUsageUpdate::apply_to` recomputes
   `total_tokens` as `input + output + cache_read + cache_creation`, discarding
   the wire's own figure -- and that recomputed sum equals
   `input + output + reasoning` whenever the cache counts happen to equal the
   reasoning count. A cached Anthropic stream would have had its output tokens
   silently doubled by the very guard meant to prevent mis-billing. The clause now
   also requires `total != cache_inclusive`, and
   `cache_totals_recomputed_by_the_stream_do_not_trigger_a_fold` pins it. The
   general lesson: a rule derived from a *wire* value must not be applied to a
   value the system recomputed itself.

2. **The internal path now records the breakdown.** The earlier entry argued the
   NULL was correct because the count genuinely did not exist at that call site.
   That was true and is no longer: the field is threaded
   `CanonicalUsage -> AiResponse -> StreamChunk::Usage -> stream accumulator ->
   TokenInfo -> AiRequestRecord -> INSERT`, so both halves of the product record
   the same thing. Nothing was under-billed before and nothing changed about
   billing -- `tokens_used` stays `input + output`, because reasoning is already
   inside output and summing it is the double-count trap.

   Two shape decisions worth keeping: `AiRequestRecordBuilder` gained a separate
   `reasoning()` setter rather than widening the public `const fn tokens()`, whose
   arity change would be source-breaking; and the INSERT's new column was
   appended as `$30` rather than renumbered, because that statement's placeholders
   are already non-sequential (`$29` appears before `$28`, since `$28` is consumed
   inside a trailing `CASE`).

   The audit query in `tests/.../ai_service.rs` did not select `reasoning_tokens`,
   so it could not have caught the NULL it was asserting nothing about. It selects
   it now.

3. **Core's migration slots are honest.** The earlier entry overstated the gap:
   core already refuses a reused slot at runtime (`verify_slot_identity` ->
   `MigrationSlotReused`), and the tombstone mechanism already covered core, since
   all 13 core migration directories share the same `build.rs`. What was missing
   was that no `.tombstone` files existed, so `ls` still lied and the failure
   surfaced at deploy rather than at commit.

   Core's one numbering gap, `infra/security/006`, was deleted pre-0.11.2 in
   `7b117a1ec` and never reached a released database -- genuinely safe, but
   indistinguishable by `ls` from the astound-034 case that would have shipped a
   silently-skipped migration. It now carries a tombstone, and a test in the
   extension unit tier walks every `crates/**/schema/migrations` failing on any
   gap without one.

   That test deliberately avoids two failure modes live in the repo at the time:
   it climbs to the repo root rather than counting ancestors (`.ancestors().nth(5)`
   in `bridge/i18n/key_coverage.rs` silently walks an empty tree when stale), and
   it **fails** rather than returning early when it finds no migration directories
   -- a test that skips itself is indistinguishable from one that passes (§6c).

---

## 2c. Thinking-budget starvation on the gemini wire — **FIXED** (core `70ec89184` → `71aeda426`; openai-chat detection `1662df939`; limits lookup `f70e3b9db`; catalog `10dcb241`)

When a client sends no `thinking` block, `wire/gemini/request.rs` sends no
`thinkingConfig` either, so Google applies its own default budget. On the 2.5
models that default is *dynamic* and can consume the whole `max_tokens`
allowance before a single visible token is emitted — the request comes back
`finish_reason: "length"` with an empty candidate, which is exactly the shape
that made the original §2 entry look like a token-accounting bug.

It is not an accounting bug: the tokens were spent, and they are now counted.
It is a request-construction defect, because a client that never asked to think
should not have its entire output budget spent thinking. The fix is to derive an
explicit `thinkingConfig` on the gemini wire rather than leaving the budget to
the provider's default. `max_thinking_budget` is already carried per model in
`services/ai/providers.yaml`, so the ceiling exists in configuration and is not
being used.

Fixed, in two steps that are worth reading as a pair. The first cut
(`70ec89184`, `e3a6ad611`) sent an explicit `thinkingConfig.thinkingBudget` when
the client sent none. Live, that switched thinking **on** for
`gemini-2.5-flash-lite`, which Google ships with thinking off: 4,373 thinking
tokens on "say hello in one word" (29 before), long enough for the upstream
connection to drop. A budget is a cap the model may fill, not a target, and
sending one changes the model's default. The landed design (`71aeda426`) sends
**no** `thinkingConfig` on the implicit path -- each model keeps Google's own
default, off for Flash-Lite, dynamic for Flash and Pro -- and only raises
`maxOutputTokens` to `min(cap, caller max_tokens + max_thinking_budget)`, so
default thinking can no longer starve the caller's text. An explicit client
`thinking` block is untouched.

Two neighbours fell out of the same measurement. The openai-chat wire had the
same starvation for the Vertex `MaaS` thinking models because
`is_reasoning_model` was a hardcoded OpenAI name-prefix list; it now also keys
on the catalog budget (`1662df939`), and the seven open reasoning models carry
one (`10dcb241`). And none of it applied to either Vertex provider until
`f70e3b9db`: the gateway resolved `ModelLimits` with the *upstream* model name,
which the catalog matcher (id + aliases) never matches, so every provider whose
ids differ from its upstream names silently had no limits at all -- no output
clamp, no budget, no reasoning ceiling. `gateway_limits_by_catalog_id.rs` pins it.

One consequence to know about: `qwen.qwen3-next-thinking` now genuinely thinks
for minutes on trivial prompts (600+ thinking deltas in the first 35 s, visible
"Wait, ..." loops) because it finally has the room. That is the model, not the
wire; harness timeouts were raised to 600 s to accommodate it.

---

## 2d. Cache tokens were double-billed, and there were two cost functions — **FIXED 2026-09-05** (core `c79b0dbee`, `fad081539`, banked in `873ed65dc`; astound `82470a29`, `9b6d87ff`)

Found by auditing everything around §2b once it closed. Each item was correct
by assumption; each is now correct by construction.

**The bug.** `OpenAI` chat, `OpenAI` responses and Gemini report `cached_tokens`
as a *subset* of the prompt count; Anthropic reports it *disjoint*. All four
adapters mapped the two as disjoint, so the gateway charged the cached slice at
the input rate **and** at the cache-read rate. The `+128c` Cerebras rows in the
morning's live run were this shape: over-billed, not credited. Six adapter
sites now subtract (`input_tokens` is exclusive of cache reads on every wire),
Gemini's stream carries the cached count it used to drop, and a per-wire test
pins buffered == streamed on all three counts.

**The shape.** `CanonicalUsage` is the only usage type — `CapturedUsage`,
`CostTokens` and the internal `TokenUsage` builder are gone, with their twelve
conversions. `ModelPricing::cost_microdollars(&CanonicalUsage)` lives in the
shared crate and prices all four billable classes on both the gateway and the
agent path, which previously priced two and fabricated a $1/$1 rate for an
unknown provider. `CanonicalUsage::billable_total()` is the one definition of
`tokens_used`; the astound admin no longer re-sums components. With `input`
exclusive of cache, the reasoning guard's additive shape is exactly
`total == billable_total + reasoning`, which also catches an additive provider
that caches — the earlier arithmetic missed those.

**The catalog.** A boot validator refuses any model on a cache-capable wire
that declares no `cache_read_per_million` (an explicit `0.0` passes), so the
subtraction cannot silently turn over-billing into $0. Gemini and Vertex carry
Google's published 25%-of-input rates; the nine MaaS partner models, Cerebras
and the e2e mock declare `0.0` with the source cited. `ModelCapabilities.reasoning`
marks the eight thinking models. Landed in three steps so committed state was
never unbootable: yaml rates, then core field + validator, then `reasoning: true`.

**The test tier.** One `db_pool_or_skip!()` in core and one `TempDb` in astound
(`tests/common`), both panicking under `CI` when the database is absent — 1,538
core sites and 664 astound sites that used to return green silently are now
declared or gated, and `lint-silent-skips.sh` refuses new ones. `repo_root()`
climbs until found. Usage builders mean a new field on the usage types breaks no
test literal. The degenerate gateway matrix went from 16 cells (one hardcoded
inbound) to 48; per-provider cache-pricing, terminal-frame-omits-usage, §1c
terminal-discipline and two property suites (SSE framing across arbitrary chunk
boundaries, stop-reason totality) were added.

**Three near-misses worth keeping.** (1) The total-based reasoning clause read a
recomputed streaming total as the additive signal whenever cache counts equalled
the reasoning count — a cached Anthropic stream would have had its output
doubled by the guard meant to prevent mis-billing. (2) The validator as first
written refused the shared test catalog and, worse, the catalog `admin setup`
generates; the gap was the embedded `default_catalog.yaml`. (3) A Gemini test
fixture missing `"role": "model"` made the whole response fail to deserialise,
and `parse_response` fell back to an all-zero usage rather than erroring — a
malformed upstream reply bills zero silently. That fallback is the one **open**
item from this sweep: `wire/gemini/response.rs:parse_response` should surface a
parse failure, not zero it.

**Proven live, 2026-09-05 18:27 UTC** on core `e951ddfe8` + astound `a8ea6652`,
`scripts/e2e-wires.sh` 220/226 cells (the six failures are DeepSeek upstream
`429` throttling and one known gpt-oss model-behaviour cell, both §3c).
The audited rows for the same Cerebras shape as the morning:

| when | `gpt-oss-120b` row | tokens_used | cost µ$ | arithmetic |
|---|---|---|---|---|
| morning, pre-fix | `146+128c/34` | 308 | 77 | 146×0.35 + 34×0.75 = 76.6 — cached slice at the input rate |
| now | `20+128c/43` | 191 | 39 | 20×0.35 + 128×0.0 + 43×0.75 = 39.25 — cached slice at Cerebras's declared 0.0 |

`tokens_used` is `billable_total` (20+128+43), the input count is the prompt
minus the cached slice, and the delta on the cached slice is exactly
128 × (0.35 − 0.0) = 44.8 µ$ per request. `o4-mini` records reasoning inside
output (448 of 481) and Gemini likewise (247 of 263); `gpt-4.1-mini` reported no
cache this run, so its delta is not demonstrable from these rows.

## 3. `openai_chat` drops reasoning **text** when streaming — **FIXED** (core `8a0f02d28`)

Buffered was fixed in `ff0778c74`: `render_assistant_message` emits
`reasoning_content` and `response.rs` parses it back (with `alias = "reasoning"`
for the providers using the shorter spelling). **The streaming delta path in
`streaming.rs` still ignores `delta.reasoning_content`.**

So a client *streaming* a thinking model gets no reasoning trace and — per §2 —
no reasoning cost either. For `qwen.qwen3-next-thinking` and
`moonshotai.kimi-k2-thinking` that is the entire product, and the proxy this
deployment is replacing renders it today. This is a visible regression for those
users, not a cosmetic gap.

Fixed: the streaming codec emits `reasoning_content`/`reasoning` deltas as the
same canonical thinking block the buffered path produces, opened before the text
block and closed on the first text or tool-call delta, with the terminal reason
untouched. Explicit `null` reasoning fields (the `MaaS` shape) open no block.

---

## 3b. Upstream `429` and `503` are retried — **FIXED** (core `082f9f80e`, `c85d5e75b`)

Recorded here as a decision rather than a defect, because the previous behaviour
(relay the upstream status straight through) was defensible and the change is a
deliberate widening of what the gateway takes responsibility for.

The gateway now retries an upstream `429` or `503`, under these constraints:

- **Only before any byte has been relayed to the client.** Once the response has
  begun, a retry would duplicate content, so the failure is relayed instead.
- **Exponential backoff**, 1s doubling to a 30s cap, with 25% jitter.
- **`retry-after` is honoured** when the upstream sends it, in preference to the
  computed backoff.
- **Bounded attempts.** There is no unbounded loop; exhausting them is a normal
  outcome, not an error path.
- **Final failure relays the upstream response verbatim**, status and body, in
  keeping with the wire-contract rule that upstream errors are not rewritten.
- **No keep-alive is needed on streams, and none is sent.** The backoff runs
  strictly before an upstream response exists, so the client has received no
  status line yet and there is no SSE stream to keep alive. Four attempts with a
  30s cap and a `retry-after` that is clamped to that cap bound the pre-byte
  wait; the ordinary worst case is under nine seconds.
- **The retry count is observable, not yet on the request row.** Each retry
  increments `gateway_upstream_retries_total` (labelled by provider and
  triggering status) and logs a structured warning with attempt and delay.
  `ai_requests` has no metadata column and the row update is a flat list of
  typed columns, so a per-request count needs a migration; the outbound layer
  exposes an `observing_retries` seam so that column can be filled without
  touching the adapters when it is added.

---

## 3c. Defects the 32-model harness pass surfaced on 2026-09-05

Each of these was found by running the real clients (`just e2e-claude-code`,
`just e2e-opencode`) and the wire matrix (`just e2e-wires`) across every model
in `/v1/models`, asserting on `mcp_tool_executions` + `ai_request_tool_calls`
rather than on transcripts. None was visible from a single-model test.

- **Empty upstream 2xx rendered as an empty turn — FIXED** (core `ab9942d0b`).
  Vertex `MaaS` intermittently answered 200 with a body carrying no choices;
  all four buffered parsers defaulted every field and the client received a
  well-formed message with `content: []`, `stop_reason: null`, 0 tokens.
  A body with no turn and no usage, a JSON array, or an object carrying
  `error` under a success status is now a 502 relaying the raw bytes, audited
  `failed` with a 512-byte excerpt.
- **Stream ends with no terminal signal, client told nothing — FIXED** (core
  `cefaea9d4`). The tap audited `stream ended without stop event` but the
  client just saw the SSE close. Each inbound now states the abort in its own
  contract (Anthropic `event: error`, chat `data: {"error":…}` then `[DONE]`,
  Responses `response.failed`). Two parsers were also swallowing upstream
  error chunks: Gemini's `{"error":…}` and `promptFeedback.blockReason`
  deserialised into an empty success, and openai-chat's `[DONE]` after an
  error chunk synthesised a clean `stop`.
- **Same-wire passthrough skips model limits — FIXED** (core `873ed65dc` +
  `b91442b68`; `passthrough_output_tokens` is shared with the translated path).
  `/v1/chat/completions` → openai-chat upstream forwards the caller's
  `max_tokens` verbatim: OpenCode's 32000 hit qwen3-235b's 16384 cap as a 400,
  and a thinking model at 1024 was starved. The translated path clamps and
  applies the reasoning ceiling; passthrough now does the same on the token
  field only.
- **Streamed usage reported as zeros on the finish chunk — FIXED** (core
  `39b6f5148`, gate debt `e269c3cc2`): the chat lane holds the terminal until
  the stream states its end and emits the contract's trailing usage-only
  chunk; the Anthropic `message_delta` now carries real output counts. With `stream_options.include_usage`, the chat inbound put an
  all-zero usage object on the finish chunk and never sent the trailing
  usage-only chunk, while the audit row had the real counts.
- **Secret scanner flags provider thinking signatures — FIXED** (core
  `873ed65dc`, `crates/infra/security/src/policy/secrets/signatures.rs`). `governance_decisions` recorded `secret_scan` findings on
  `contents[1].parts[0].thoughtSignature` (Gemini's opaque signed blob the
  client must echo back). Warn-only here; in production, where all four
  stages enforce, that is a denial of every Gemini thinking continuation.
  The entropy detector now skips provider signature fields; pattern detectors
  still run on them.
- **Schema reinstall re-runs an applied migration — FIXED** (core `2c320a462`:
  baseline rows are now written in the same transaction as each extension's
  structural DDL, so an interrupted install can never leave tables
  installed-and-unstamped; `interrupted_install.rs` pins it).
  A second install over an installed schema executed `ai` migration 003 again
  (`column "user_id" named in key does not exist`); three cli-tests pin it.
- **Catalog limits the upstreams reject — FIXED** (`b52dea63`/`dabdc749`,
  `5a197531`): vertex-gemini-2.5-flash claimed a 32768 thinking budget (Google
  caps it at 24576), qwen3-235b and both `MaaS` gpt-oss models claimed output
  caps above what Vertex enforces. A client asking for the advertised maximum
  got 400 instead of a clamp.

Two observations that are not defects, recorded so nobody re-derives them:
Vertex `MaaS` enforces a per-project concurrency quota on `deepseek.v3.2`
("throttled due to too many concurrent requests"); the retry path carries a
single client through it but three concurrent 32-model sweeps exhaust four
attempts, so run the harnesses serially. And Vertex `MaaS` refuses a named
`tool_choice` on the two gpt-oss models and `required` on them too, while
accepting `required` on qwen, glm, kimi and deepseek; the wire matrix records
the rung each cell settled on.

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
- **Truncation mid-tool-call is not covered by either matrix.** In flight as
  part of workstream A4, with §1c — fixed in core `c161d5d81` (cells), `d316add7c` (Anthropic passthrough), `479b22c1e` + `59e68ea5c` (Responses renderer).
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

## 6j. Out of scope by decision — not defects to be picked up

These were considered in this session and deliberately left. They are kept
because "nobody looked at it" and "somebody looked at it and said no" are
different states, and only the second one is safe to leave alone.

| item | why it is out of scope |
|---|---|
| §4 Gemini URL image degradation | Correctly deferred. Fixing it means network I/O inside a synchronous wire codec, or a layer change one level up. Real work, not a patch, and the warning makes the degradation visible. |
| §6a `just prepare` deletes the cache on failure | Recovery is one `git checkout -- .sqlx`. Annoying, understood, cheap to live with. |
| §6b `migrate-repair --apply` corrupts rows | Left as documented rather than fixed. The operational rule — never run `--apply` without reading the recorded name first — is enough while the tool has one operator. |
| A native Gemini inbound adapter | Gemini clients are served today through the OpenAI-compatible surface. A third inbound surface is a large amount of wire code for no user currently asking. |
| `bridge/profile.rs:115` hardcode | Known, single-valued, and correct for every deployment that exists. Parameterise it when a second value appears, not before. |
| Host model-limit projection | The bridge does not project per-model `max_output_tokens` into host configs. Hosts pick sane defaults and the gateway is authoritative anyway. |

---

## 7. Priority

| # | Item | Status | Why this rank |
|---|---|---|---|
| 1 | §1c audit of `anthropic` + `openai_responses` terminal signals, and running the 24 offline cells | IN FLIGHT (A4) | Two of four wires had a live silent-drop; the other two are unaudited |
| 2 | §2c gemini thinking-budget starvation | IN FLIGHT (A1) | A client that never asked to think can lose its whole output budget to thinking |
| 3 | §3 streaming `reasoning_content` | IN FLIGHT (A2) | Visible regression vs the proxy being replaced, for the models whose purpose is reasoning |
| 3= | §3b upstream 429/503 retry | IN FLIGHT (A3) | A transient upstream rate limit currently surfaces to the user as a failed turn |
| 4 | §6b `migrate-repair` | Out of scope (§6j) | Silently corrupts the ledger; mitigated by procedure while the tool has one operator |
| 5 | §5 truncation-mid-tool-call coverage | IN FLIGHT (A4) | The one terminal-signal shape neither matrix tests |
| 6 | §6a `prepare` cache deletion | Out of scope (§6j) | Cheap to live with; recovery is one command |
| 7 | §4 Gemini URL images | Out of scope (§6j) | Correctly deferred; needs a layer change, not a patch |

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
- Reasoning-token accounting is verified on the gemini wire and confirmed
  absent-upstream on Vertex MaaS (§2). Nothing further is owed there.
- Workstreams A1 through A4 are mid-flight; their markers above are unresolved.
- `e2e-opencode.sh` check 6 is **red**, and correctly so — it is failing on §2,
  not on the wire bug. It has deliberately not been loosened to pass.
