# Native adapter acceptance

Run from Astound. These commands make no paid provider calls and never modify the
native capability registry:

```sh
just evals-native-fixture-build
just evals-native-acceptance /absolute/path/to/native_adapter_fixture systemprompt-evaluator:local claude-code candidate,baseline
```

The exporter is the `native_adapter_fixture` example in the core scheduler test
package. Locate it under the effective Cargo target directory's
`debug/examples/`. Its source is fingerprinted by the Astound build coordinator.
Use `all` for the client and/or scenario argument to expand a run. Client values
are `claude-code`, `open-code`, `codex`, and `hermes`. Scenario values are bounded
and cannot contain executable commands.

The exporter invokes the production adapter's configuration, arguments, exact
version parser, and output normalizer. Its synthetic target identity exists only
to materialize those contracts; it does not invoke production admission or
register a target. The native harness resolves the image to its immutable digest
and records the actual executable SHA-256 and parsed version. Runs use a fresh
internal Docker network containing only the native client and a deterministic
provider/MCP fixture, with bounded time, output, memory and process counts.
The harness maps its host UID/GID (1000 in the observed local workspace) so it
can own its temporary home without privileged ownership changes. The standalone
native probes use tester UID 1001. This difference must be reconciled with the
production supervisor's mapped identity before claiming full environment proof.

Each case retains `plan.json`, native stdout/stderr, normalized output, and the
provider's received request journal. The journal contains synthetic credentials
and fixture data, not real provider secrets. A combined `report.json` records
failures without discarding evidence. Cleanup checks Docker's live container list
and removes the owned network. A Docker inspection error does not establish
successful cleanup.

| Scenario | Required observation |
| --- | --- |
| candidate | Skill bytes returned through an actual native tool call |
| baseline | Candidate skill sentinel absent; same configuration/argv/frozen settings digest |
| mcp | Actual fixture `tools/call` received |
| credentials | Tool evidence does not expose the parent's upstream credential |
| forbidden | Unavailable tool cannot create a workspace marker |
| network | Network request cannot reach fixture outside the authorized native provider route |
| failure | Provider failure cannot produce completed evidence |
| cancel | Cancellation after a provider request cannot produce completion; cleanup confirmed |
| attempt-bound | At most one provider request |
| judge, suggestion | Native write request cannot modify workspace |

Standalone provider fixtures establish native protocol and isolation behavior.
They do **not** establish production gateway reservation, spend attribution,
failed-spend retention, or persisted request-accounting parity. Consequently the
report always declares `production_gateway_metering_verified: false`. Native
capability enablement additionally requires the production gateway proof and
review. Synthetic provider usage is explicitly labelled in each request record.
The existing network-none probes separately test native sandbox read/write and
parent-process credential isolation without any provider traffic.
