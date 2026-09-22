# sp-dev: every bridge sign-in fails with 429 from `session-pat`

**Status:** open — requires a configuration change on the host Astound operates.
**Found:** 2026-09-22. **Severity:** total sign-in outage on `sp-dev.astound.digital`.

## Symptom

Astound Bridge sign-in fails on every attempt:

```
Sign-in failed: auth: gateway returned status 429 Too Many Requests from session-pat
```

The message names `session-pat`, which reads like a rejected personal access token. It is
not. `session-pat` is the bridge's label for the endpoint it called,
`POST /v1/auth/bridge/session-pat`. The credential was never examined — the request was
refused by the gateway's HTTP rate limiter before it reached the handler. That handler can
only answer 400, 401 or 500; it has no 429 path at all.

## Evidence

A single request from a machine whose IP had never contacted the host:

```
$ curl -sI https://sp-dev.astound.digital/v1
HTTP/2 429
x-ratelimit-limit: 300
x-ratelimit-remaining: 0
retry-after: 36
```

Zero tokens remaining on **first** contact. A per-client bucket would have returned 300. So
the bucket is not per client — the whole instance shares one. `retry-after: 36` at 100
tokens/second means it is roughly 3,600 requests in debt, i.e. permanently saturated. Under
that condition no sign-in can ever succeed, and no amount of waiting or retrying helps.

## Cause

`sp-dev` runs behind `nginx/1.20.1`.

The gateway resolves a caller's address by a deliberately strict rule: if the immediate
socket peer is **not** listed in `server.trusted_proxies`, the forwarded headers
(`X-Forwarded-For`, `X-Real-IP`, `Fly-Client-IP`, `CF-Connecting-IP`) are ignored entirely
and the peer's own address is used. This is correct — an untrusted hop's headers are
forgeable, and honouring them would let any client pick its own rate-limit bucket.

On `sp-dev`, nginx's peer address is not covered by `server.trusted_proxies`. So every
request from every user resolves to nginx's address, they all share one rate-limit bucket,
and ordinary inference traffic on `/v1/messages` keeps it empty. Bridge sign-in shares that
bucket and is starved out.

This does not reproduce on a local install because the local profile sets
`rate_limits.disabled: true` — the limiter is never constructed there.

## Fix

On the `sp-dev` host, in the deployed `profile.yaml`:

```yaml
server:
  trusted_proxies:
    - 127.0.0.0/8
    - ::1/128
    - 172.16.0.0/12      # REPLACE: the range nginx's connection actually arrives from
rate_limits:
  gateway_per_second: 100   # currently unset, so it silently runs this same default
```

Restart the server for the profile to reload.

### Determining the right CIDR

The value must cover the address nginx connects **from**, as the application sees it — not
the address nginx listens on, and not the client's address.

- nginx in a container on the default docker bridge: `172.16.0.0/12` (covers `172.17.x.x`).
- nginx in the same compose network: that network's subnet (`docker network inspect <net>`).
- nginx on the host, proxying to a published port: `127.0.0.0/8` may be enough, but with
  docker's userland proxy the peer is often the bridge gateway — list both.
- **nginx on the host network with a public address:** that address must be listed
  explicitly. The private ranges above will not cover it, and this case currently produces
  no warning in the logs.

If unsure, the server logs the peer it saw when it rejects forwarded headers from a private
peer:

```
ignoring forwarded client-IP headers from untrusted private peer; if this server runs
behind a proxy, add the peer's range to server.trusted_proxies
```

Search the logs for `untrusted private peer` and use the `peer_ip` field. Note this warning
is not emitted when the peer has a public address — see the core fix below.

### Confirming the fix

```bash
curl -sI https://sp-dev.astound.digital/v1 | grep -i x-ratelimit
```

Before: `x-ratelimit-remaining: 0` for every caller, permanently.
After: a client that has not just made hundreds of requests should see `x-ratelimit-remaining`
close to `300`, and it should differ between two clients on different networks.

### Emergency stop-gap

Set `rate_limits.disabled: true` in the rendered profile the container mounts, and restart.

The `rate_limits:` block is **not** optional — `Profile.rate_limits` carries no
`#[serde(default)]`, so an instance whose profile lacks it cannot deserialise its profile and
does not boot. An operator who cannot find the block is reading a different file from the one
the container loads (a bundle template, or a directory other than the one the profile-dir
variable points at), not a profile that genuinely omits it.

There is **no usable environment override**. `RATE_LIMIT_DISABLED` is only read by
`Profile::from_env` (`crates/shared/models/src/profile/from_env.rs:22`), the cloud/subprocess
boot path that builds a profile out of the environment; an instance that loads a `profile.yaml`
never calls it. It is also matched as `== "true"` (`from_env.rs:193`), so the `=1` spelling
would be silently ignored even on the path where it does apply.

Disabling removes rate limiting from **every** route, not just the gateway, and with it the
protection against credential-stuffing and request floods on the public endpoints. It is a
measure for hours while the correct CIDR is found, not a configuration — the `trusted_proxies`
fix leaves that protection in place.

## Follow-up in the product

The configuration above resolves the outage, but three defects in the gateway made it
possible and are being fixed in core:

1. Bridge sign-in shares one rate-limit bucket with inference traffic, so a busy instance
   locks users out of authenticating. Auth is being given its own budget.
2. The limiter's per-user keying never engages on `/v1`, because authentication happens
   inside each handler rather than in middleware the limiter can see. Every gateway request
   is therefore bucketed by IP, which also means a single NAT'd office shares one bucket
   even when `trusted_proxies` is correct.

   This one is documented rather than fixed, deliberately. The two obvious repairs are both
   worse than the status quo: verifying credentials before the limiter makes an
   unauthenticated flood *more* expensive to absorb, and keying on an unverified bearer token
   hands an attacker unlimited buckets by rotating a header. The route worth taking, when it
   is picked up, is structural — extract the gateway handlers' authentication into a
   `ContextLayer` the way every other router already does with `with_auth`. The existing
   ordering in `with_rate_limit` then solves it for free: the per-address governor stays the
   outer layer, so a flood is refused before any verification happens, while identity keying
   and the DB-backed `global_user_rate_limit` sit inside it and only ever see requests that
   already paid the cheap check. The catch is that the gateway's handlers each authenticate
   slightly differently — PAT, bridge JWT, session header — so the layer must cover every
   inbound shape or the ones it misses silently lose their identity bucket. That is a real
   refactor and does not belong bolted onto an incident fix.
3. A proxy with a public address that is missing from `trusted_proxies` produces no warning,
   which is why this configuration ran undiagnosed.
