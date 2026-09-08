# opsx-eval

A small, self-contained module for evaluating OpenSpec artifacts with an
LLM-as-a-judge on Amazon Bedrock. It has one reusable **judge core** and two
subcommands over it:

- **`eval`** — the primary tool for developing our rules/prompts. It generates N
  candidate artifacts in memory from a generator manifest + case context, judges
  each, and prints an aggregate **variability** report (pass rate, score
  distribution, per-dimension means, recurring findings). Nothing is written to
  disk unless you pass `--save`.
- **`judge`** — a thin CLI over the same core that judges one already-produced
  file. This is the "live" path (e.g. validating an artifact a real OpenSpec run
  produced).

This directory is the module **source**. On project initialization it is copied
to `.cursor/.project/eval`, next to the shared `.cursor/.project/` tooling.
Credentials come from the shared `.project/.env` via `../lib/load-env.mjs` (the
same loader the rest of the tooling uses, resolving the same way in both
locations). To port the module elsewhere, swap that one side-effect import in
`run.mjs`.

## How it works

```
evalsuite/<schema>/
  mappings.yaml             # live-reference -> stub/redirect registry (project-context, jira, ...)
  judge/<artifact>.md       # judge rubric (contract + rubric); frontmatter declares `context:` and `sources:`
  judge/reasoning.md        # judge routing-pass procedure for this schema
  generator/<artifact>.md   # generation MANIFEST; frontmatter points at real command/skills/schema
  generator/contract.md     # generation harness contract for this schema
  generator/reasoning.md    # generation planning-pass procedure for this schema
  fixtures/<case>/          # a golden case in the OpenSpec change-folder convention
    .context/               #   input CONTEXT packet (kept apart from golden outputs)
      initial-context.md    #     raw sources (BRD/SRD, meeting notes) — discovery grounding only
      initial-context.<flow>.md #  per-flow overlay appended to the base packet
    discovery.<flow>.md     #   golden artifacts ("as if the stage already ran")
    specs/<capability>/spec.md
    fsd/<feature>.md
```

For the full design and the maintenance rules (what to update when the project
changes), see the `opsx-eval` skill (`.cursor/skills/opsx-eval/SKILL.md`) — the
design + actualization contract.

- **The generator artifact file is a manifest, not prose.** Its frontmatter points
  at the REAL project assets a Cursor subagent would use, so editing a
  command/skill/template is reflected in eval automatically:
  - `command:` — a real workflow command (e.g. `commands/opsx-propose.md`), path
    relative to the generator manifest file.
  - `skills:` — a list of skill names (resolved under `paths.skills_root`) or
    relative paths to `SKILL.md` files.
  - `schema_artifact:` — `<schema>/<artifact>` (e.g. `feature/discovery`); pulls
    the schema's per-artifact template + instruction (what `openspec instructions`
    synthesizes) from `paths.openspec_root`.
  - `sources:` — extra rules/templates inlined verbatim (unchanged behaviour).
- **Live references are neutralized via `mappings.yaml`.** Commands/skills mention
  infrastructure Bedrock cannot reach (project-context MCP, Jira, the
  `openspec` CLI, `AskUserQuestion`/`TodoWrite`). Each match gets a substitution
  note appended to its block (a stub, or a redirect to the CONTEXT that carries
  the `.context/initial-context` packet). `--prompt-only` prints which references
  were substituted.
- **Context is declarative per artifact.** Each `judge/<artifact>.md` frontmatter
  lists the `context:` it needs; the resolver pulls those from the case folder.
  `discovery` reads `.context/initial-context`; downstream artifacts read sibling
  artifacts (`fsd` reads `discovery` + `specs`). The same declaration drives
  generation inputs.
- **`eval` generate → judge in one process.** Generation runs at
  `temperatures.generate` (> 0, so the N samples differ); the judge runs at
  `temperatures.judge` (0, for stable scoring). The candidate never touches disk.
- **The generation prompt never sees the golden artifact.** The golden is a
  judge-only reference.

## Prerequisites

- Node >= 20
- `AWS_BEARER_TOKEN_BEDROCK` in `.project/.env` (the same shared key
  project-context uses)
- Bedrock access to the generator/judge models selected by the active mode in
  `config.yaml` (`dev`: `google.gemma-4-31b` generator + `openai.gpt-oss-120b-1:0`
  judge; `release`: `openai.gpt-oss-120b-1:0` + `mistral.mistral-large-3-675b-instruct`),
  all in `us-east-1`. Gemma 4 needs the `bedrock-mantle` endpoint, reachable with
  the same `AWS_BEARER_TOKEN_BEDROCK` API key.

## Install (once per machine)

```bash
cd .cursor/.project/eval
npm install
```

## Run

```bash
# Develop rules: generate N candidates and judge each, then read the aggregate.
# Concurrency defaults to 20 lanes (config.yaml eval.concurrency); override if needed.
npm run eval -- --artifact fsd --dir store-locator --runs 10

# discovery is flow-aware when a case ships discovery.<flow>.md goldens:
# store-locator currently auto-runs ba + sa (3 runs/flow in dev).
npm run eval -- --artifact discovery --dir store-locator

# Save every candidate + verdict + aggregate under .results/ (gitignored):
npm run eval -- --artifact fsd --dir store-locator --save

# Judge one already-produced file (live path). --context grounds traceability:
npm run judge -- --artifact fsd --file <path-to-fsd.md> --context <case-or-change-dir>

# Preview the assembled prompt without calling Bedrock:
npm run eval -- --artifact fsd --dir store-locator --prompt-only
```

`--dir` / `--context` resolve as an absolute path, a cwd-relative path, or a case
name under `evalsuite/<schema>/fixtures/`. Default `--schema` is `openspec-feature`.

### Run modes (`--mode`)

Judging is the expensive part, so `--mode` selects a cost preset that switches the
generator, judge, run count, and judge samples in one flag. Modes are defined in
`config.yaml` under `modes:`; the default is `default_mode` (`dev`).

| Mode | Generator | Judge | Runs × samples | ~Cost/batch* | Use for |
|---|---|---|---|---|---|
| `dev` (default) | `gemma-4-31b` | `gpt-oss-120b` | 3/flow × 1 | varies | Fast/cheap iteration; lenient judge, no self-consistency. |
| `release` | `gpt-oss-120b` | `mistral-large-3` | 10/flow × 3 | varies | Pre-release gate; strict judge, median + majority-vote criticals. |

*Rough for the `store-locator` discovery case.

```bash
# Cheap dev loop (default — no flag)
npm run eval -- --artifact discovery --dir store-locator

# Pre-release gate
npm run eval -- --artifact discovery --dir store-locator --mode release

# Trial a candidate generator/judge without editing config
npm run eval -- --artifact discovery --dir store-locator --gen-model google.gemma-4-26b-a4b
```

Explicit `--runs N`, `--judge-samples N`, `--gen-model <id>`, and
`--judge-model <id>` all override the selected mode. Develop in `dev`, then
validate in `release` before trusting a change. See the `opsx-eval` skill, "Run
modes (cost control)" for the rationale.

### Delivery-scope flows

Discovery cases may define per-flow overlays and goldens:

```text
.context/initial-context.md      # shared base context packet
.context/initial-context.ba.md   # functional/FSD flow overlay
.context/initial-context.sa.md   # integration/ISD flow overlay
.context/initial-context.both.md # negative split-gate overlay
discovery/initial.ba.md          # positive functional golden — the first draft
discovery/final.ba.md            # the same artifact after its review gate
discovery/final.sa.md            # positive integration golden
```

An artifact stored as a directory of states keeps the draft and the settled
artifact apart: a golden resolves to the draft (what generation must produce) and
a context to the settled state (what a downstream artifact receives), with the
draft falling back to the settled state on flows that carry no review session.
State names come from `states:` in `config.yaml`.

`eval --artifact discovery --dir <case>` automatically runs only flows with a
golden for them. The `both` overlay is intentionally a negative
split-gate fixture for complex combined asks and ships no golden; run it manually
with `--flow both --prompt-only` (or `--runs 1 --flow both --save` when inspecting
a sample) and confirm the output is a stop-and-ask that recommends splitting the
change (per the `opsx-propose` split gate), not a finished discovery artifact.

Goldens should cite project-context evidence using the real inbox anchor shape
returned by MCP, e.g.
`.cursor/.project/project-context/inbox/meeting_notes/workshop.md:12-28`, or
`openspec/specs/<capability>/spec.md:<lines>` for baseline specs. Do not cite the
eval-only `.context/initial-context.md` file in artifact output.

Note: Gemma 4 models are served only on the OpenAI-compatible `bedrock-mantle`
endpoint, so `lib/bedrock.mjs` routes any `google.gemma-4*` id over HTTP (Bearer
API key). Everything else uses the Converse API.

### discovery on the `judge` CLI

`judge --artifact discovery` needs discovery's primary sources, so it requires
`--context <case>`: the fixtures (`.context/initial-context` + baseline specs) then supply
them. Without `--context` it is rejected (a live change folder does not carry the
raw sources). This is also how you run the **golden ceiling check** — score the
golden against its own rubric to confirm the ideal passes ~100%:

```bash
node run.mjs judge --artifact discovery \
  --file evalsuite/openspec-feature/fixtures/store-locator/discovery.md \
  --context store-locator
```

## Cost (rough, by mode)

Pricing at time of calibration: gpt-oss-120b ~$0.15 / 1M input and ~$0.60 / 1M
output tokens; Mistral Large 3 ~$0.50 / 1M input and ~$1.50 / 1M output tokens.

- **`dev`** (5 × 1, gpt-oss both sides): ~$0.04 per `store-locator` discovery batch.
  Cheap enough for dozens of runs a day.
- **`release`** (20 × 3, gpt-oss gen + Mistral judge): ~$0.53 per batch. The judge
  (60 calls) dominates; most generation cost is output tokens.

Costs scale with prompt/output size, so treat these as order-of-magnitude.

## Bedrock Region Split

OpenSpec eval is standalone: it generates and judges from fixture/context blocks
inside this module and does not call project-context or its reranker at runtime.
That means project-context can stay in Frankfurt
(`PROJECT_CONTEXT_BEDROCK_REGION=eu-central-1`) for retrieval/rerank support,
while eval can use a different region to access the tested generator/judge pair.
Both use the same `AWS_BEARER_TOKEN_BEDROCK`.

Eval model placement is owned by `config.yaml`, not `.project/.env`. This keeps
the tested model pair versioned with the eval and avoids environment drift. Model
availability depends on region; the current judge, Mistral Large 3, is available
in `us-east-1` / `us-west-2`, so the eval config uses `us-east-1`.

## Configuration: `OPENSPEC_ROOT`

`paths.openspec_root` (the schema templates/instructions + live `specs/`/`changes/`)
is sourced from the `OPENSPEC_ROOT` env var when set, else the config default
(`../templates/openspec`, the shipped scaffold). On project init the initializer
rewrites `OPENSPEC_ROOT` in `.project/.env` to the real project `openspec/`. See
the `opsx-eval` skill, "Path resolution".

## Adding a new artifact or schema

No code changes. Drop files following the convention:

- `evalsuite/<schema>/judge/<artifact>.md` — rubric, with frontmatter `context:`
  (grounding ids) and optional `sources:` (paths relative to the file).
- `evalsuite/<schema>/generator/<artifact>.md` — the generation manifest, with
  frontmatter `command:` / `skills:` / `rules:` / `schema_artifact:` (real assets)
  and optional `sources:`. `rules:` carries only the process-affecting rules a
  real subagent gets by glob (not behavioural `alwaysApply` rules) — see
  the `opsx-eval` skill, "Reference & attachment model".
- `evalsuite/<schema>/generator/contract.md` and
  `evalsuite/<schema>/generator/reasoning.md` — the schema-level harness contract
  and inline planning-pass procedure. They structure how the generator uses the
  real assets; they do not replace the schema artifact's domain rules.
- `evalsuite/<schema>/fixtures/<case>/` — a golden case: the `.context/`
  input packet (`initial-context*.md`) plus the golden artifacts named by the
  `context:` ids and the artifact itself.
- If the flow depends on a NEW live service/tool, add an entry to
  `evalsuite/<schema>/mappings.yaml` so it is neutralized. Verify with
  `--prompt-only`.

## Layout

```
run.mjs             CLI entry point: eval / judge subcommand dispatch
config.yaml         Bedrock models/region, temperatures, eval runs/concurrency, thresholds, paths
lib/                config, resolve, references (mappings), bedrock, prompt, judge core, report
evalsuite/          openspec-<schema>/: mappings.yaml + judge/generator role files + golden fixtures per schema
```

The design + actualization contract lives in the `opsx-eval` skill
(`.cursor/skills/opsx-eval/SKILL.md`); read it before extending.
