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
The harness checks its mapped UID/GID against the owned workspace metadata,
matching the Unix `ContainerLaunch` runtime-user calculation (1000 locally).
Production rejects a root-owned workspace. The standalone network-none probes
use tester UID 1001; the production 1001 fallback applies only outside Unix.
Archive files use the supervisor's 0700 executable/0600 data modes.

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


## Production gateway wire replay

After retaining native request and response artifacts:

```sh
just evals-native-gateway-build
just evals-native-gateway-replay /absolute/path/to/native_gateway_replay /absolute/path/to/native-acceptance/run /absolute/path/to/gateway-report.json
```

This helper uses the core API integration fixture database and actual
`GatewayService::dispatch`, inbound/outbound codecs, audit journal and accounting
settlement. A local wiremock upstream returns the exact retained response bytes.
It checks fixture wire usage (11 input / 7 output tokens), persisted priced spend
(25 microdollars at configured synthetic rates), provider failure status, and
retention of spend after a post-response accounting fault. Unknown pricing must
remain an explicit resolver error. Available native token summaries are compared
with persisted wire totals; disagreement fails the replay rather than changing
provider accounting to match the client.

This replay does not enroll an evaluation session or exercise the live execution
reservation path. Its report therefore keeps live native gateway budget
admission and native target enablement unverified. The replay's synthetic API key
is supplied explicitly and never contacts a paid provider.


## Live execution capabilities and gateway accounting

```sh
just evals-native-live-build
just evals-native-live /absolute/path/to/native_adapter_fixture /absolute/path/to/native_live_gateway systemprompt-evaluator:local claude-code candidate,baseline
```

The optional live fixture issues a real execution capability and session from
an enrolled fixture worker and a leased repository experiment. The actual native
client sends each request to the isolated provider container. That container
forwards the request over a private Unix socket into the core gateway helper;
the native client cannot mount or access that socket. The helper authenticates
the execution capability and session, then invokes `GatewayService::dispatch`
with the real evaluation reservation repository. A loopback deterministic
upstream provides the fixture wire response. The native response is returned
only through the actual gateway codec and accounting flow.

Each successful provider response must settle its own reservation for 25
microdollars (11 input and 7 output tokens at synthetic configured rates), and
completed native usage summaries must agree when the client reports them.
Finalization injects a journaled accounting failure for one actual completed
native request, repeats recovery settlement, and requires failed status with
the original usage and reservation retained without extra spend.
The final report compares total settled spend and outstanding reservation
bounds with the real budget account. Attempts without complete usage remain
explicit outstanding reservations; the fixture never invents zero-cost usage.
Forged credentials, session substitution and excessive output requests must
leave reservation/accounting state unchanged. Expired leases and rotated
execution tokens must fail authentication. Normal shutdown revokes the worker
and waits for the helper process to exit. Interruptions retain a recovery-needed
marker if orderly gateway finalization could not be confirmed.

`live_execution_reservation_verified` is a per-case observation requiring actual
native attempts and these accounting checks. It does not enable a capability.
The repository setup injects an explicit deterministic-fixture admission
collaborator; no production allowlist is bypassed or modified. Exact image,
executable, native configuration and tool evidence remain in the same report.
The helper uses synthetic revision lineage for its accounting experiment,
which is separate from a publication-quality baseline/candidate experiment.
Baseline/candidate native configuration comparisons normalize only freshly
issued execution token, session and execution identifiers.

The active deadline in these scenarios is enforced by the acceptance harness,
using the adapter plan's limit. `harness_deadline_enforced` does not claim that
`EvaluatorSupervisor` ran the process. Supervisor lifecycle tests and final
mapped-user/image review remain separate acceptance evidence.
