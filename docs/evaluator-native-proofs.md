# Reviewed native evaluator proof registration

The trusted registry is empty until native isolation and actual execution-token gateway acceptance pass. Runtime configuration and fixture validation cannot register a target. Desktop installation remains independent of automated evaluation.

A registration consists of three retained UTF-8 JSON documents embedded with `include_str!` in core's `capabilities::proofs::REVIEWED`: a version-1 `NativeProofManifest`, a version-1 `NativeProofReport` for isolation, and another for gateway metering. The declaration also embeds the SHA-256 of the exact manifest bytes. The target's two evidence digests must hash the exact corresponding report bytes. Documents have a 1 MiB bound; unknown fields, unsupported versions, failed/missing contracts, malformed hashes and mismatched bindings reject registration. Merely constructing or validating a public manifest does not add it to the registry.

All three documents bind the client, platform, architecture, exact executable and adapter versions, executable SHA-256, image config SHA-256, and core/Astound source-content SHA-256 values. Source digests identify the sources used for acceptance, including any uncommitted test-stage changes; retain the source file manifest used to calculate each digest. Reports additionally retain a nonempty map from safe relative artifact names to SHA-256 values. Keep the sanitized underlying artifacts with their reports so reviewers can reproduce those hashes; report validation does not fetch artifacts from a network. Do not retain real credentials or raw private session tokens in committed artifacts.

Isolation requires passing `pinned_executable`, `configuration_credentials`, `forbidden_tools`, `network`, `cancellation_cleanup`, `attempt_bound`, `output_bound`, `failed_evidence`, `matched_environments`, `judge`, and `suggestion` contracts. Metering requires passing `authenticated_execution_session`, `atomic_reservation`, `native_usage_parity`, `failed_spend`, `unknown_pricing`, `attempt_output_cost_bounds`, `stale_lease`, `revocation`, and `idempotent_settlement`. A partial/interrupted report or a fixture-only result cannot supply a passing native contract. Harness-enforced deadlines must remain identified as such; they do not prove execution by the production supervisor.

Local images use the exact `sha256:<config-ID>` reference. Their manifest has `repository_manifest_digest: null` and the target's `image_digest` equals the config ID. Repository images use `repository@sha256:<manifest-digest>`: retain that manifest digest separately, set target `image_digest` to it, and retain the exact config ID it resolves to. A Docker config ID must never be represented as a repository manifest digest. Startup inspects the actual image config ID before probing executable bytes and version; all three must match the reviewed registration. Neither registration method requires a deployment or image publication.

After reviewing all required native and live-gateway results against the final immutable image and sources, retain the reports, add the reviewed embedded declaration, and run admission, registry and runtime verification contracts. Discovery and admission use the same platform/architecture match. Baseline and candidate must still share client/version/image/configuration/model; only the skill bundle may differ. No registration is enabled by this documentation or by the fixture test manifests.

## Local evidence packaging

Run the full native harness with the actual live gateway exporter and an immutable image reference:

```sh
EVAL_DOCKER_BIN=/usr/bin/docker node scripts/evaluator/native-acceptance.cjs \
  /absolute/path/native_adapter_fixture sha256:EXACT_IMAGE_CONFIG_ID all all \
  /absolute/path/native_live_gateway
node scripts/evaluator/native-proof-package.cjs \
  .build/native-acceptance/RUN_DIRECTORY codex /absolute/new/proofs/codex
```

Repeat packaging for `claude-code`, `open-code`, and `hermes`. The output directory must not exist. Packaging requires every scenario and its matching actual gateway finalization to pass. It checks fixed executable/image/adapter identities, matched environments, native usage parity, reservation coverage, stale/revoked capabilities, repeated settlement retaining failed spend, and explicit output, unknown-pricing and cost rejection probes. Missing fields from older reports fail closed; rerun acceptance rather than adding them by hand. Packaging never edits the reviewed registry.

The harness retains source manifests before execution and compares source content and exporter binary hashes after the run. Manifests include tracked and unignored untracked files, file modes, deletions and symlink targets; they contain hashes, not source contents or credentials. The source identity is the **pre-registration acceptance source**, not a claim that subsequent reviewed registration leaves the whole repository hash unchanged. Retain those manifests with the reports. The separately recorded exporter hashes identify the actual binaries; operators must build them from the captured sources before acceptance. Adding reviewed proof documents and registry declarations requires a subsequent admission/runtime validation gate.

The packaged `native-evidence.json` is a sanitized projection of observed harness assertions and the original report digest. It excludes capability tokens, raw request bodies, diagnostics and private environment bytes. Keep original local reports for review; only the sanitized packaged documents are candidates for retention in source control. Cancellation, a deliberately delayed provider response, and deliberately oversized native output run through the same `ContainerLaunch`/`ContainerExecution` start, poll and cancel primitives used by the production supervisor. The output case lowers the supported artifact cap to 64 KiB and delivers a larger deterministic response through the native CLI; packaging requires observed overflow, core rejection and acknowledged cleanup. The exporter uses a fixture-only verifier after the harness has checked the exact image, executable bytes and parsed pinned version. This does not claim a complete production scheduler lifecycle or enable admission before reviewed registration.

Run the packaging failure contracts with `node --test scripts/evaluator/native-proof-package.test.cjs`. These synthetic validation tests establish rejection behavior only and cannot enable a native target.
