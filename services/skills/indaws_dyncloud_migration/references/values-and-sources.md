# The variables, and where each value comes from

Nothing here is a value to copy: it is **where to read it**. Per-environment data (uuid, FQDN) and
platform data (the manager's URL) change per target and over time, so they are looked up at the
moment they are needed and never written into a skill.

## How to obtain them (you do not need access to the manager's database)

The navigation chain of the **DynCloud MCP** answers two of the four in one call, and it is the
intended way: `list_environments` returns, per environment, its **`uuid`** and its **`aud`** — the
audience the manager signs — alongside `id`, `name`, `stage_type` and `state`.

```text
list_my_companies                     → company_id
list_projects     { company_id }      → project_id
list_environments { project_id }      → { id, name, uuid, primary_url, aud, stage_type, state }
                                          ^^^^                          ^^^
                                          ENV_UUID                      ALLOWED_AUD
```

Take `uuid` and `aud` **verbatim** from that answer. `aud` is exactly what the verifier compares
against, so copying it removes any doubt about trailing dots, subdomains or the environment's
public URL differing from its audience.

The other two need no lookup: `LINEAGE_NAME` is `docker` for this mode, and `BACKEND_URL` is the
manager's own public URL — the host of the MCP endpoint you are already talking to. Confirm it with
the documentation MCP rather than reconstructing it from a cluster hostname.

If the environment does not appear in `list_environments`, it is not exposed to MCP
(`mcp_enabled = false`) — that is a manager-side setting, not something to work around from here.

**Which repository and which branch?** Not in `list_environments`. Each environment is a *branch*
of its project's code repository, and the branch is normally the environment's own name. Get the
repository URL from the environment's form in the manager, or ask whoever owns the project — and
confirm the branch exists before cloning. Do not infer it from the project name.

Driving these tools — the navigation chain, the five mandatory location args, the JIT grants a
`production` environment will demand — is **`common-skills:dyncloud-mcp`**. Read it if you have not
used this MCP before: none of the calls above work without the five location args, and on a
production stage several categories are gated behind a grant you have to request first.

## Required


| Variable | What it is | Where the value comes from |
|---|---|---|
| `DYNCLOUD_AUTH_BACKEND_URL` | Public URL of Dyncloud Manager. The environment calls it to fetch the signing keys (JWKS). | Ask the documentation MCP for the manager's public URL, and cite it. Do **not** reconstruct it from a cluster hostname. |
| `DYNCLOUD_AUTH_ENV_UUID` | Identifies **this** environment to the manager. | `list_environments` → `uuid`, copied verbatim (see above). It is the `uuid` of that `cloud.environment` record. |
| `DYNCLOUD_AUTH_LINEAGE_NAME` | Which signing-key lineage to ask for. | **`docker`** for this mode. Not a guess: the manager holds separate lineages, and Docker environments verify against the Docker one. See GOTCHA 2 in `SKILL.md`. |
| `DYNCLOUD_AUTH_ALLOWED_AUD` | Expected audience of the tokens; the verifier rejects anything else. | `list_environments` → **`aud`**, copied verbatim (see above). Never retype it from the environment's URL: the two can differ. |

## Recommended

| Variable | What it is | Value |
|---|---|---|
| `DYNCLOUD_API_MAX_BODY_BYTES` | Request-body cap of the API dispatcher. | The module defaults to **4 MiB** and answers **413** above it. Kubernetes environments get a much larger cap from the chart; set the same figure here so file transfers and standard Odoo imports behave identically on both runtimes. Ask the documentation MCP for the value in force on the chart rather than picking one. |

## Must stay UNSET

| Variable | Why not |
|---|---|
| `DYNCLOUD_AUTH_BACKEND_SPKI` | Setting it **turns off normal certificate validation** (`verify_mode = CERT_NONE`) and replaces it with a pin on the SHA-256 of the **entire DER certificate** — not the SubjectPublicKeyInfo, despite the name; the code says so in its own comment. With a Let's Encrypt certificate on the manager, that pin breaks at every renewal. Left empty you get full CA validation, and the module records its own pin on first contact (TOFU) in its table, immutable afterwards. |
| `DYNCLOUD_AUTH_SIGNING_KEYS_PATH` | Kubernetes-only. There the public keys arrive as files through an ExternalSecret and the module runs in **disk mode**. Its mere presence on a Docker environment switches the module to that mode and **disables the JWKS cron** the environment depends on. |

## Optional, with sane defaults — leave alone unless asked

`DYNCLOUD_AUTH_REQUIRE_IP_BINDING`, `DYNCLOUD_AUTH_ALLOW_MFA_SKIP`,
`DYNCLOUD_AUTH_ALLOW_PORTAL_USERS`, `DYNCLOUD_AUTH_MAX_JWT_BYTES`,
`DYNCLOUD_AUTH_DB_ALLOWLIST`, `DYNCLOUD_AUTH_REFRESH_INTERVAL_DAYS`,
`DYNCLOUD_AUTH_ACCEPTED_CA_FINGERPRINTS`, `DYNCLOUD_API_MASK_SALT`.

`DYNCLOUD_API_MASK_SALT` deserves a note: its default derives a salt **per database**, which keeps
masked values from being correlatable across databases. Setting it makes the salt process-global —
only appropriate where cross-database correlation is wanted on purpose.

`DYNCLOUD_AUTH_DB_ALLOWLIST` is the one worth considering as hardening: it restricts which database
names the connector will serve. Default is "all". Only set it if you are certain of the database
name, since getting it wrong locks the environment out.

## They are not boot-time settings

The natural assumption is that these seed a first sync and can then be removed. They cannot.

`_read_bootstrap_config()` is called from the **JWKS refresh cron**, on every tick — not only at
install. It reads the companion addon's bootstrap file if present, otherwise these environment
variables, and **`ir.config_parameter` is deliberately never consulted**. What the module persists
is the *result* (the fetched keys, the TOFU pin, the lineage), never the configuration.

Remove the variables after the first successful sync and the environment keeps working — on cached
keys — while the refresh returns early every tick with a single `JWKS_REFRESH_FAIL` audit entry.
The failure only becomes visible when the manager rotates its signing key and every token starts
being rejected, with nothing obvious pointing back at the removal.

Comment them in the compose file so the next person does not tidy them away.
