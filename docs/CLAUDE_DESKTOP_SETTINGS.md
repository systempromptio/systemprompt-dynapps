# Claude Desktop settings audit

Audited 142 top-level settings against [Anthropic’s configuration reference](https://claude.com/docs/third-party/claude-desktop/configuration). The machine-readable catalog in core is the validation source. Unset operator options preserve Desktop behavior. Provider-specific credentials are not embedded in the Bridge profile.

| Setting | Type | Minimum Desktop | Disposition | Reason |
|---|---|---|---|---|
| `inferenceCustomHeaders` | object | 1.8089.0 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `inferenceSessionLifetimeSec` | integer | 1.14271.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `inferenceCredentialHelper` | string | 1.2581.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `inferenceCredentialHelperWindows` | string | 2.2553.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `inferenceCredentialHelperArgs` | string[] | 2.110.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `inferenceCredentialHelperTtlSec` | integer | 1.2581.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `inferenceCredentialHelperTimeoutSec` | integer | 1.8089.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `inferenceCredentialHelperSilentRefreshEnabled` | boolean | 1.10628.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `egressProxyUrl` | string | 1.44121.1 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `egressProxyPacUrl` | string | 1.44121.1 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `coworkVmIpv6Enabled` | boolean | 1.52386.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `userContentRendererUrl` | string | 1.24012.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `inferenceProvider` | enum | 1.2581.0 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `inferenceCredentialKind` | enum | 1.8555.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `inferenceAnthropicApiKey` | string | 1.8089.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceBedrockRegion` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceBedrockBaseUrl` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceBedrockServiceTier` | enum | 1.5186.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceBedrockBearerToken` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceBedrockSsoStartUrl` | string | 1.6259.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceBedrockSsoRegion` | string | 1.6259.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceBedrockSsoAccountId` | string | 1.6259.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceBedrockSsoRoleName` | string | 1.6259.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceBedrockProfile` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceBedrockAwsDir` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceBedrockAwsCliPath` | string | 1.13576.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceFoundryResource` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceFoundryBaseUrl` | string | 2.110.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceFoundryApiKey` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceFoundryTenantId` | string | 1.9255.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceFoundryClientId` | string | 1.9255.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceFoundryAuthFlow` | enum | 1.19367.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceGatewayBaseUrl` | string | 1.2581.0 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `inferenceStreamIdleTimeoutSec` | integer | 1.44121.1 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `inferenceGatewayApiKey` | string | 1.2581.0 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `inferenceGatewayAuthScheme` | enum | 1.3036.0 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `inferenceGatewayOidcAuthFlow` | enum | 1.25927.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `inferenceGatewayOidc` | object | 1.6889.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `inferenceIdpAuthFlow` | enum | 2.7032.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `inferenceIdpOidc` | object | 2.7032.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `modelDiscoveryEnabled` | boolean | 1.8089.0 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `modelPrefer1mContext` | boolean | 1.28929.0 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `inferenceModels` | object[] | 1.2581.0 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `defaultModelEffort` | enum | 2.110.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `alwaysStartWithDefaultModel` | boolean | 2.110.0 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `inferenceModelPricingEnabled` | boolean | 1.37937.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `inferenceModelPricingMultiplier` | number | 1.37937.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `inferenceModelPricing` | object[] | 1.37937.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `modelCatalogEnabled` | boolean | 2.110.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `modelCatalogUrl` | string | 2.110.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `inferenceVertexProjectId` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceVertexRegion` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceVertexBaseUrl` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceVertexOAuthClientId` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceVertexOAuthClientSecret` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceVertexOAuthScopes` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceVertexOAuthLoginHint` | string | 1.12603.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceVertexWorkforceAudience` | string | 1.10628.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceVertexWorkforceUserProject` | string | 1.10628.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceVertexWorkforceAuthFlow` | enum | 1.25927.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceVertexWorkforceOidc` | object | 1.10628.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `inferenceVertexCredentialsFile` | string | 1.2581.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `disableDeploymentModeChooser` | boolean | 1.3834.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `disableDeepLinkRegistration` | boolean | 1.6889.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `builtinBrowserEnabled` | boolean | 2.2553.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `builtinBrowserDefaultDomainPolicy` | enum | 2.2553.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `builtinBrowserAllowedDomains` | string[] | 2.2553.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `builtinBrowserBlockedDomains` | string[] | 2.2553.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `chatTabEnabled` | boolean | 1.13576.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `chatAdvancedFileAnalysisEnabled` | boolean | 1.14271.0 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `isClaudeCodeForDesktopEnabled` | boolean | 1.2581.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `sshHostAllowlist` | string[] | 1.40609.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `sshClientPath` | string | 1.46388.1 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `sshTransport` | enum | 1.52386.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `coworkTabEnabled` | boolean | 1.9659.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `userPluginMarketplacesEnabled` | boolean | 1.37937.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `userPluginUploadsEnabled` | boolean | 1.37937.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `disabledBuiltinTools` | string[] | 1.2581.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `disableBundledSkills` | boolean | 1.15962.0 | default | Enable normal public web tools and bundled skills. |
| `skillCreationEnabled` | boolean | 1.25927.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `scheduledTasksEnabled` | boolean | 2.110.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `keepAwakeEnabled` | boolean | 2.7032.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `builtinToolPolicy` | object | 1.8089.0 | default | Enable normal public web tools and bundled skills. |
| `autoModeEnabled` | boolean | 1.10628.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `disableBypassPermissionsMode` | boolean | 1.46388.1 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `toolSearchEnabled` | boolean | 1.21459.0 | operator-configurable | Enable through the signed fleet policy only after native-tool routing is supported. |
| `skipWebFetchPreflight` | boolean | 1.37937.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `allowedWorkspaceFolders` | object[] | 1.2581.0 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `blockReadsOutsideWorkingDirectories` | boolean | 1.46388.1 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `coworkEgressAllowedHosts` | string[] | 1.2581.0 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `requireCoworkFullVmSandbox` | boolean | 1.2581.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `organizationInstructions` | string | 1.37937.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `claudeAiImport` | object | 1.10628.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `microsoftAuthBroker` | enum | 1.19367.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `isDesktopExtensionEnabled` | boolean | 1.2581.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `isDesktopExtensionSignatureRequired` | boolean | 1.2581.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `managedMcpServers` | object[] | 1.2581.0 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `mcpPersistentAlwaysAllowEnabled` | boolean | 1.24012.9 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `mcpScheduledTaskApprovalLifetimeDays` | integer | 2.7032.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `isLocalDevMcpEnabled` | boolean | 1.2581.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `allowedPluginMcpServers` | object[] | 2.2553.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `deniedPluginMcpServers` | object[] | 2.16120.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `mcpToolTimeoutSec` | integer | 1.37937.0 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `deploymentOrganizationUuid` | string | 1.2581.0 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `disableEssentialTelemetry` | boolean | 1.2581.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `disableNonessentialTelemetry` | boolean | 1.2581.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `disableNonessentialServices` | boolean | 1.2581.0 | bridge-owned | Bridge owns authentication and transport hardening; keep the signed gateway identity authoritative. |
| `disableAutoUpdates` | boolean | 1.2581.0 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `autoUpdaterEnforcementHours` | integer | 1.2581.0 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `updateViaUpdatesHost` | boolean | 1.26832.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `relaunchEnforcementHours` | integer | 1.40609.0 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `configRecheckIntervalMinutes` | integer | 1.46388.1 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `disableLocalConfigCache` | boolean | 2.7032.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `otlpEndpoint` | string | 1.2581.0 | bridge-owned | Authenticated Bridge telemetry transport and server-bound identity are authoritative. |
| `otlpProtocol` | enum | 1.2581.0 | bridge-owned | Authenticated Bridge telemetry transport and server-bound identity are authoritative. |
| `otlpHeaders` | object | 1.2581.0 | bridge-owned | Authenticated Bridge telemetry transport and server-bound identity are authoritative. |
| `otlpAuthMode` | enum | 1.30096.1 | bridge-owned | Authenticated Bridge telemetry transport and server-bound identity are authoritative. |
| `otlpHeadersHelper` | string | 1.30096.1 | bridge-owned | Authenticated Bridge telemetry transport and server-bound identity are authoritative. |
| `otlpResourceAttributes` | object | 1.5354.0 | bridge-owned | Authenticated Bridge telemetry transport and server-bound identity are authoritative. |
| `otlpDesktopLogLevel` | enum | 1.9255.0 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `otlpContentCapture` | enum[] | 1.15962.0 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `otlpTracesEnabled` | boolean | 1.22209.0 | default | Chosen fleet default for predictable 1M operation, updates, file analysis, or metadata diagnostics. |
| `chatSessionRetentionDays` | integer | 1.52386.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `coworkSessionRetentionDays` | integer | 1.52386.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `codeSessionRetentionDays` | integer | 1.52386.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `sessionRetentionHold` | boolean | 1.52386.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `inferenceMaxTokensPerWindow` | integer | 1.2581.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `inferenceTokenWindowHours` | integer | 1.2581.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `endUserAttribution` | boolean | 1.25927.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `deploymentDisplayName` | string | 1.24012.0 | operator-configurable | Use the deployment name from the signed fleet policy; do not hard-code client branding in core. |
| `deploymentDisplaySubtitle` | string | 1.24012.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `disableConfigDeprecationWarnings` | boolean | 1.40609.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `banner` | object | 1.7196.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `disableFeatureDiscovery` | boolean | 1.21459.0 | operator-configurable | Explicit deployment option; preserve current behavior when unset. |
| `orgPluginSettings` | object[] | 1.8089.0 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `allowedPluginMarketplaces` | object[] | 1.17377.1 | bridge-owned | Resolved from trusted gateway, identity, model, connector, or workspace sources. |
| `bootstrapEnabled` | boolean | 1.10628.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `bootstrapUrl` | string | 1.10628.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `bootstrapOidc` | object | 1.10628.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `bootstrapHeaders` | object | 1.32885.1 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `bootstrapHeadersHelper` | string | 1.32885.1 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |
| `trustBootstrapDelivery` | boolean | 1.26832.0 | inapplicable | Not used by the Bridge gateway deployment; do not emit. |

## Public web access

The generated policy explicitly writes `coworkEgressAllowedHosts: ["*"]`,
`builtinToolPolicy: {"WebSearch":"allow","WebFetch":"allow"}` and
`disableBundledSkills: false`. Leaving egress unset is not unrestricted:
Desktop permits only the inference endpoint. WebSearch executes at the
inference provider and is independent of the Cowork egress list. The AI service's
historically named `google_search_enabled` flag enables provider-native search
for Anthropic, OpenAI and Gemini; it is enabled for the configured providers.
An explicit CLI/environment egress restriction still takes precedence over the
default, so remove a previous `loopback`/`127.0.0.1` override when testing.

## Reproduction and acceptance

1. Install the updated Bridge, sync with the gateway and repair the Desktop
   profile. macOS requires approving the generated profile in System Settings;
   generating or opening the file does not install it. On Windows verify the
   effective machine policy: an existing HKLM Claude policy replaces HKCU.
2. Fully quit Desktop, reopen it, and start a new Cowork task. Existing tasks
   can retain the previous model/context. Run Bridge doctor and inspect
   Desktop's configuration window. Confirm the model catalog contains only
   available Claude models, uses the `[1m]` variants, and has no Fable entry.
3. Ask: “Use WebSearch to find Anthropic’s current Commercial Terms, then
   WebFetch the official page and give me its URL and current effective date.”
   Confirm actual tool calls and citations. A model's prose claiming a block is
   not proof of a network policy. Record any failing tool's HTTP status and
   provider request ID. Do not use a shell workaround.
4. In a fresh task, progressively load a reproducible text fixture past 300K
   and then 700K input tokens. Count input plus cache-read plus cache-creation
   tokens, rather than summing conversation billing rows. Inspect the actual
   outgoing model/context variant and upstream errors. Verify there is no
   repeated compaction loop around 170–220K tokens.
5. Test large serialized requests separately. The Bridge/gateway buffered
   inference request cap is 32 MiB (other buffered routes retain 8 MiB); a 1M context window does not raise this byte
   cap. Include tool schemas, images and JSON framing in the measurement.
6. Invoke Dynapps Odoo and DynBrain, idle through credential refresh, then
   invoke both again without reconnecting. Compare provider refresh errors
   with Bridge session errors. Verify a native task remains one UI session
   after context compaction.
7. Repeat steps 1–6 on macOS and Windows. Render tests verify equivalent
   policy encodings; they cannot prove either native app applied a profile.

## Diagnostics and rollout

The profile sends OTLP protobuf to the authenticated loopback `/otel` relay
using its inference credential. Desktop application logs use HTTP JSON and
are decoded separately from Cowork protobuf traces. Gateway ingestion filters metadata keys and
removes log bodies, span/status text, arbitrary attributes and metric names.
Authenticated user/session identity replaces emitter-supplied identity.
`otlpContentCapture` must remain empty. No dedicated external collector is
required for the existing logging repository ingestion path.

Doctor compares fleet defaults and signed operator overrides against installed
policy and warns about settings unsupported by the detected Desktop build.
Tool-search wire preservation is covered by a regression fixture. The direct
Anthropic Haiku API accepted deferred tools (HTTP 200), and a separate search
probe executed one server WebSearch successfully (HTTP 200). Requests carrying
native web/tool-search definitions select the Anthropic API route; ordinary
inference continues to use Vertex. Tool search is enabled in the fleet profile.
The installed Desktop version and an actual 300K/700K task must be verified
before claiming the compaction incident is resolved. Staging and the live
Dynapps endpoint must be verified separately, including downloaded binaries.
