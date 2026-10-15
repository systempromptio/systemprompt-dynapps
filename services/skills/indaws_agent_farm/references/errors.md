# Errors in plain words

`scripts/farm.py` already turns each API error into the sentence in the third column. This table is
for when you need to decide what to do next. Say it in the user's own words and language; the code
is for you, not for them. Never read a code aloud unless they ask what went wrong technically.

Every error the farm returns carries `error.retryable`: `true` means repeating the same request can
work (the team is at capacity, the database or a store was busy), `false` means it cannot until
something changes. `farm.py` already adds "it is temporary: the same request can be repeated in
about N seconds" when it is `true`. When you explain an error, use it: a `true` is "inténtalo de
nuevo en unos segundos" (wait the `Retry-After` seconds, repeat once or twice, then tell the
operators); a `false` is never repeated unchanged. `task_not_terminal` is `false` on purpose: it is
not repeated blindly, you wait for the task (`--wait`).

| Code | HTTP | What it means for the person | What to do |
|---|---|---|---|
| `token_expired` | 401 | their key's validity ended | stop; they need a new key from whoever set them up. Do not retry |
| `token_invalid` | 401 | the key is wrong, revoked or incomplete | check `AGENT_FARM_TOKEN` holds the whole key (no spaces); otherwise a new key. Do not retry |
| `insufficient_scope` | 403 | the key is valid but lacks this permission | say which action it was and ask them to request that permission (`farm.py api <operation>` says which one it needs) |
| `tenant_not_accepted` | 400 | a request tried to set the team | a bug in what was sent: remove the field. Never ask the user |
| `malformed_json` | 400 | the request body was not valid JSON | a bug in the skill; report it |
| `payload_invalid` | 422 | a field is missing or has a wrong shape | read the message for the field name; fix it or ask the user for the missing value |
| `agent_or_prompt_required` | 422 | neither or both of agent and free prompt | pick one: agent if the request fits the catalogue, otherwise prompt |
| `input_required` | 422 | an agent was named but no text to work on | ask what it should work on |
| `prompt_not_enabled_for_tenant` | 422 | their team may not run free-text tasks (`--prompt`), only catalogue agents | do not retry with text; `farm.py` already lists the agents after the error. Offer the one whose description fits (`--agent NAME --input TEXT`; ask for the repository if it needs one) |
| `agent_not_found` | 404 | the agent does not exist or is not available to this team | list the agents and offer the closest |
| `workspace_required` | 422 | this agent works on a repository and none was given | ask for the repo address (or deduce it, see `intents.md`) |
| `workspace_required_for_resume` | 422 | a follow-up of a repo task must name the repo again | send the same repo again |
| `scope_invalid` | 422 | the optional label has capitals or odd characters | lowercase, digits, hyphens; or drop the label |
| `limits_invalid` | 422 | a limit is not a valid quantity (a `cpu` or `memory` that is not like `1` or `2Gi`) | fix the value, or drop the limit and use the agent's defaults |
| `limits_exceed_tenant_policy` | 422 | asked for more time or memory than the team allows | the message carries the maximum; offer to retry within it |
| `quota_exceeded` | 429 | a team usage limit was reached | wait the number of seconds given, or cancel something running; do not loop |
| `credential_type_invalid` | 422 | the kind of saved access is not supported | use `git_token` for a repo token |
| `credentials_ref_out_of_tenant` | 422 | the access name has `/` or `..` or belongs elsewhere | use a plain saved-access name |
| `credential_not_in_agent_manifest` | 422 | that agent is not allowed to use that saved access | use one of the names `agents <slug>` lists, or another agent |
| `credentials_not_found` | 404 | no saved access with that name, or it is a teammate's | list theirs; offer to save one (`intents.md`) |
| `session_not_found` | 404 | no earlier conversation like that, or it is not theirs | list their recent tasks and use one that says it can be continued |
| `plugin_name_reserved` | 422 | a plugin in their marketplace has the name of a built-in one | rename it in their marketplace; `plugins` lists the reserved names |
| `plugin_catalogue_unavailable` | 422 | the farm could not check plugin names | retry in a minute |
| `marketplace_egress_not_granted` | 422 | the agent may not reach the site hosting their marketplace | only GitHub and `gitlab.dynapps.be` are usually possible; otherwise ask the farm's operators |
| `agent_plugin_not_sent` | 422 | asked for `<plugin>:<agent>` without listing that plugin | add it to the plugin list |
| `cron_invalid` | 422 | the repeat pattern is not five valid fields | rebuild it from what the user said (`schedules.md`) |
| `timezone_invalid` | 422 | unknown time zone | use an IANA name like `Europe/Madrid` |
| `recurrence_invalid` | 422 | dates or counts contradict each other | read the message; usually an end before the start |
| `task_invalid` | 422 | the task inside the schedule would never be accepted | read the message; fix the task part |
| `not_found` | 404 | does not exist **or is not theirs** (the farm does not tell which) | check the id; a teammate's task looks exactly like this |
| `task_not_terminal` | 409 | asked for the result of a task still running | check status; wait |
| `task_has_no_result` | 409 | finished without producing a result | read the logs |
| `task_not_cancellable` | 409 | already finished | tell them it already ended and how |
| `result_expired` | 410 | results are deleted after 30 days | nothing to recover; offer to run again |
| `schedule_finished` | 409 | the schedule reached its end date or run count | it cannot be resumed; offer to create a new one |
| `schedule_not_active` | 409 | `schedule fire` on a paused or finished schedule | the script says which: paused → resume it first; finished → offer a new one |
| `schedule_overlap` | 409 | `schedule fire` while the previous run is still going and overlap is `skip` | wait for that run (`schedule runs <id>`), or offer `--overlap allow` on a new schedule |
| `database_busy` | 503 | the farm's database was busy and a wait ran out; nothing was done | repeat the same request after a few seconds (`Retry-After`) |
| `credential_too_short` | 422 | an access used as an environment variable is shorter than 6 characters, too short to hide in the agent's output | ask for a longer value and register it again (`credentials add`); never echo it back |
| `state_invalid` | 422 | `--state` names a state that does not exist | use one or several of the states `farm.py tasks --help` lists, separated by commas |
| `cluster_unavailable` | 503 | the execution platform is unreachable | the request is kept and retried by the farm; check status in a few minutes. Do not launch a second one |
| `object_store_unavailable` | 503 | storage is unreachable | retry later; logs and results are unaffected |
| `secret_store_unavailable` | 503 | the secret store is unreachable | retry later |
| `cursor_invalid` | 422 | a list position is stale | list again from the start |
| `input_too_large` | 422 | the agent's instructions plus the text exceed the limit | the message carries how many characters to cut; shorten the text |
| `credential_type_not_allowed` | 422 | a task with no catalogue agent got an access that is not a `git_token` of theirs | use a git token for the repository, or an agent that declares the other access |
| `credential_forge_mismatch` | 422 | the access was saved for another host than the repository | save a token for this repository's host (`--repo`); a token never travels to another host |
| `credential_value_invalid` | 422 | the secret has the wrong shape for its kind (an `mcp_url` is an `https` address) | check the kind and the value; nothing was saved. Never echo the value back |
| `credential_write_conflict` | 409 | two administrators saved the same new shared access at once | save it again |
| `credential_unavailable` | 422 | an access the agent names does not exist, or cannot be used by whoever the agent belongs to | `credentials list` (and `--scope shared`); save it first or fix the name |
| `shared_credential_required` | 422 | a team agent used a personal access where the farm demands the team's (an MCP with its own `url`, a variable) | use `shared:<name>` (an administrator saves it); never a personal one |
| `shared_credential_not_allowed` | 422 | `shared:<name>` in a personal agent or a task | personal agents and tasks use the person's own accesses; or make it a team agent (administrator) |
| `env_var_invalid` | 422 | the variable name for an access is not `^[A-Z][A-Z0-9_]*$` or is one the farm uses (`ANTHROPIC_*`, `AGENT_*`, `MCP_*`, `CLAUDE_*`, `PATH`, `HOME`…) | pick another name |
| `mcp_server_name_duplicate` | 422 | two MCP servers share a name (the agent's and the task's count together) | rename one |
| `mcp_server_url_invalid` | 422 | the MCP `url` is not `https` or carries a user, password or key | remove the secret from the address and save it as an access (`mcp_token`, or `mcp_url` with no `url`) |
| `mcp_credential_type_mismatch` | 422 | the MCP access kind does not match the entry: `mcp_url` when there is no `url`, `mcp_token` when there is; a house server name takes an `mcp_token` | save the access with the right kind (`credentials add --type …`) |
| `agent_slug_taken` | 409 | a platform agent or one of the team's has that name | pick another name, or edit the existing agent |
| `agent_slug_reserved` | 422 | the name has a `:` or is `__prompt__` | pick another name; `<plugin>:<agent>` is for plugin agents |
| `agent_read_only` | 403 | tried to change or delete a platform agent | cannot be done from here; build a team or personal agent on it (`plugin_agent`) |
| `agent_in_use_by_schedule` | 409 | a schedule still launches that agent | `schedules`, delete or change it, then retry |
| `agent_id_invalid` | 422 | the `<plugin>:<agent>` is malformed | letters, digits, `.`, `_`, `-` on each side; `plugins` lists the real ones |
| `agent_not_in_image_plugin` | 422 | the plugin exists but defines no such agent | the message lists what it has; use one of those |
| `agent_tools_outside_ceiling` | 422 | the plugin agent declares only tools the farm forbids, so it could never run | pick another agent; the farm's operators decide the ceiling |
| `agent_egress_not_allowed` | 422 | the agent asks to reach more sites than the team's policy grants | `egress` only narrows; ask for fewer destinations or the operators for more |

Statuses with no code worth reading: `404` on anything means "does not exist or is not yours";
`403` is a missing permission; `429` is a limit; `503` is temporary. Network errors and timeouts
reach you as a plain sentence from `farm.py`, never as a code.

A launch that failed on the network may still have happened. Before launching again, look at
`tasks --limit 5`; or send the same `correlation_id` both times (`intents.md` → Launch).
