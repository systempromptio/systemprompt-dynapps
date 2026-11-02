# dynwork-relay

The DynWork relay: a systemprompt extension that lets DynWork Mobile drive a DynWork desktop with no inbound port
and no VPN. The desktop and the phone each hold one WebSocket to this relay. Everything they say to each other is
end-to-end encrypted with Noise by `@dynwork/remote` (the fork's `dynwork/packages/remote`, whose
`relay-protocol.md` is the contract this crate implements). The relay routes opaque envelopes by peer id, and it
never holds a key that opens them.

## Routes

All routes sit under `/api/dynwork/relay/v1`. The base path is under `/api/` on purpose: core's session middleware
treats every other path as a tracked site page and would mint an anonymous session, and record the URI, for each
socket and claim. Under `/api/` it skips them.

| Method and path | Caller | Credential | Answer |
| --- | --- | --- | --- |
| `GET /desktop` (WebSocket) | DynWork desktop | `Authorization: Bearer <DynWork JWT>`, audience `bridge` or `api`; `X-Dynwork-Peer: <peer id>` | `101`, then envelopes |
| `GET /device` (WebSocket) | DynWork phone | `Authorization: Bearer <device credential>` | `101`, then envelopes |
| `POST /pairings` | desktop | DynWork JWT | `201 {pairing_id, token, expires_at}` |
| `POST /pairings/claim` | phone | body `{token, peer_id, name?}` | `201 {device_id, peer_id, credential}` |
| `GET /devices` | desktop | DynWork JWT | `200 [{id, kind, peer_id, name, created_at, last_seen_at, online}]` |
| `DELETE /devices/{id}` | desktop | DynWork JWT | `204` for a phone; `409 desktop_not_revocable` for a desktop; `404` for a device that is not the caller's or is already revoked |

Credentials are read from the `Authorization` header or the JSON body only: never from a cookie, a path or a query
string, so none ends up in a URL or an access log. A refused socket is a plain HTTP error before the upgrade.

**Which JWT is a desktop.** Signature, issuer and audience are not enough. Core's anonymous site session is a
first-party JWT too, with audiences `web`, `api`, `a2a` and `mcp` and a real anonymous user row. So the relay accepts
only a token that:

- has the audience `bridge` or `api`;
- has the user type `user` or `admin`;
- carries no `act` (delegation), no `plugin_id` and no `purpose`.

That is what core's bridge device link mints for the DynWork desktop: audiences `bridge` and `mcp`, the person's own
permissions, no purpose. A token that expired, even inside core's 30 s leeway, is refused. Errors are JSON
`{"error": <key>, "message": <text>}`:

| Status | `error` | When |
| --- | --- | --- |
| 400 | `bad_request` | a missing or malformed `X-Dynwork-Peer`, a device name over 64 characters |
| 401 | `unauthorized` | a missing, forged, expired or wrong-audience token; a token that is not a signed-in person's; an unknown or revoked credential |
| 409 | `device_limit`, `peer_taken`, `desktop_not_revocable` | the user is at the device limit; the peer id is already one of the user's live devices; `DELETE` on a desktop |
| 410 | `pairing_used`, `pairing_expired`, `pairing_unknown` | the pairing token cannot be redeemed (`pairing_unknown` also covers a value that is not a pairing token) |
| 422 | (axum) | a claim body that is not `{"peer_id": <32 hex>, "name"?: <string>}` |
| 429 | `too_many_pairings` | five open pairings already |
| 503 | `overloaded` | the relay is at its tunnel limit (`Retry-After: 1`) |

### Pairing

1. The desktop calls `POST /pairings` and gets a **pairing token**: `dwp1_` plus 32 random bytes in base64url,
   opaque, 48 characters, and valid 5 minutes. The relay keeps only its SHA-256. It carries no identity, name or
   email, so the QR and any log stay free of them. A user holds at most five open pairings, counted and inserted
   under the user's row lock.
2. The desktop puts it in the QR beside the end-to-end pairing secret.
3. The phone calls `POST /pairings/claim` with `{token, peer_id, name}` in the body, never in the URI. The relay
   finds the row by the token's hash, locks it and stamps it in one transaction, so a token is redeemed once. It
   answers with a **device credential**: `dwr1_` plus 32 random bytes, of which it too stores only the SHA-256. A
   refused claim (device limit, peer taken) leaves the token claimable until it expires.

The pairing token is **not** the end-to-end pairing secret. That secret is checked by the desktop inside the Noise
handshake and never reaches the relay, so a malicious relay cannot pair a device of its own. The relay authorises
sockets with instance credentials only.

The phone sends its **peer id**, not its key: the first 16 bytes of SHA-256 of its static X25519 key, 32 lowercase
hex digits. The relay never sees a key.

### Device credentials from other sign-in paths

`GET /device` checks the bearer token against a chain of `auth::DeviceVerifier`s. The first verifier that recognises
the token decides, and a token none recognises is refused. The relay ships `CredentialVerifier` for `dwr1_`
credentials. A later sign-in path adds its verifier with `RelayStateBuilder::device_verifier`, and the socket code does not
change. The mobile sign-in's `MobileCredentialVerifier` (`dwm1_`) is the second link of the chain by default.

## Mobile sign-in (path 1)

`mobile_auth/` signs the DynWork phone app in the way the desktop signs in: through the system browser and this
instance's own login, so no passkey is needed. Its routes sit under `/api/dynwork/mobile/v1`, beside the relay's; the
extension mounts both under `/api/dynwork` (`extension_router`). The contract, with every answer, is section 11 of the
fork's `dynwork/packages/sp-client/PROTOCOL.md`.

| Route | Auth | Does |
| --- | --- | --- |
| `GET /discovery` | none | tells the app path 1 is offered, and its paths |
| `GET /authorize` | browser session cookie | consent page; signed out: `307` to `/admin/login?redirect=…` |
| `POST /authorize/decision` | browser session cookie | Allow: `303` to `dynwork://auth/callback?code&state`; Deny: `?error=access_denied&state` |
| `POST /token` | none (code + PKCE verifier) | a `dwm1_` mobile credential, a one-hour JWT, the device and the account |
| `POST /refresh` | `dwm1_` credential | a fresh JWT |
| `GET /session` | mobile JWT | the account and device, while the device is live |
| `POST /revoke` | `dwm1_` credential | the phone signs itself out |

- **Session.** Only the `access_token` cookie core's sign-in sets is read, verified as a session cookie, and it must be
  a browser session of a person (no anonymous visitor, `act`, plugin scope or purpose). An `Authorization` header is
  never read on the consent routes.
- **Consent.** Showing the page opens a grant (`dynwork_relay_mobile_grants`) bound to the person, the PKCE challenge,
  the `state` and the device name, keyed by the SHA-256 of a `dwq1_` request token that only the page's form carries.
  That token is the CSRF guard: a decision for an unknown token, another person's grant, or an answered or expired one
  is refused. At most 10 grants per person are open at once.
- **Code.** `dwa1_`, ten minutes from Allow, single use, bound to the S256 challenge. Unknown, used, expired and
  wrong-verifier codes all answer `400 invalid_grant`; a wrong verifier burns the code; a code presented again revokes
  the device its first use created.
- **Credential.** The exchange creates a `phone` device row with the `dwm1_` credential's SHA-256, in the same
  transaction that redeems the code, under the device limit. It is listed and revoked like a paired phone. Signing in
  again with the same peer id revokes that peer's previous phone row.
- **JWT.** Minted by the instance's token authority, one hour, audience `dynwork-mobile` only and purpose
  `dynwork-mobile`: core's session middleware, the bridge and MCP routes and the relay's desktop routes refuse it. The
  device id is its session id, so `GET /session` refuses it once the device is revoked. `/refresh` re-reads the
  account and refuses a suspended or deleted one.
- **Page.** Server-rendered, no script, every value escaped, in the instance's branding (`branding.display_name`,
  `theme_color` and `logo_light` from `web/config/theme.yaml`, each dropped for a default when unsafe). It forbids
  framing, allows forms to post only here and to redirect only to `dynwork:`, is never cached and sends no referrer.
  All copy is in `mobile_auth/copy.rs`.

## Routing

- **Envelope.** `version (1) ‖ to (16) ‖ from (16) ‖ seq (4) ‖ ciphertext (1..65535)`, big-endian, at most 65572
  bytes, one per binary WebSocket message. The relay reads `version` and `to`, overwrites `from` with the sender's
  authenticated peer id, and forwards the rest unchanged. It never parses, logs or keeps the ciphertext beyond the
  ring buffer.
- **Within one user.** Everything is keyed by `(user, peer id)`. A socket reaches only sockets of its own user, and
  only the other kind of end: phones talk to desktops, desktops to phones. Anything else is dropped silently.
- **Order.** Envelopes from one sender to one recipient arrive in the order sent (`relay-protocol.md` rule 3). Each
  recipient has one bounded queue, filled under the hub's lock.
- **Many phones per desktop.** A desktop socket carries every phone of its user; the desktop tells them apart by
  `from`.
- **Ring buffer.** An envelope for a known peer of the user that is not connected is held in the ring of the desktop
  end of the pair: at most 256 envelopes, 4 MiB and 120 s, the oldest evicted first. When the peer attaches, what was
  held for it is delivered, in order, before anything live. Delivery stays best effort: the ends resume over a new
  channel and resend what was lost.
- **Replacement.** A second socket for the same `(user, peer)` replaces the first, which is closed with `4001`.
- **Closing without loss.** A tunnel that closes leaves the routing table before it sends its close frame, so what
  arrives during the close handshake is held. Whatever its queue still held is put back at the head of the ring of
  the desktop end, ahead of anything held since. A slow consumer or a stuck writer loses only what had reached its
  socket. Every write has a deadline (the heartbeat timeout, 2 s for the close frame), so a peer that stops reading
  cannot hold a tunnel, its permit or the drain.

### Presence

The relay tells a user's desktops when a phone of that user attaches or goes away. Phase 04's desktop starts its
resume timeout from this signal instead of guessing. A presence message is a binary WebSocket message the **relay**
writes, with the envelope's offsets and a reserved version:

| Offset | Size | Field | Value |
| --- | --- | --- | --- |
| 0 | 1 | `version` | `0xFF` (relay control; never a client envelope) |
| 1 | 16 | `to` | the desktop's peer id |
| 17 | 16 | `phone` | the phone's peer id |
| 33 | 1 | `event` | `1` connected, `2` disconnected |

It is exactly 34 bytes and carries ids only.

- **When it is sent.** A desktop gets `connected` when a phone of its user attaches, and one `connected` for each
  phone already online when the desktop itself attaches. It gets `disconnected` when a phone's tunnel ends: the phone
  closed, the transport failed, the heartbeat or lifetime expired, a revocation, a slow consumer.
- **When it is not sent.** A phone socket replaced by a newer one is not reported as gone; the newer one sends
  `connected` again.
- **Order.** Presence goes through the desktop's queue with the envelopes, so the desktop sees a phone's last
  envelope before its `disconnected`.
- **Spoofing.** A client cannot forge presence: the relay drops every envelope whose version is not `1`, and
  `0xFF` included.

A desktop must check byte 0 before decoding an envelope: `0x01` is an envelope, `0xFF` is presence, and anything
else is ignored.

## Limits

| Limit | Default | On breach |
| --- | --- | --- |
| Devices per user (desktops and phones, not revoked) | 10 | `409 device_limit` |
| Open pairings per user | 5 | `429 too_many_pairings` |
| Pairing token lifetime | 300 s | `410 pairing_expired` |
| Envelope size | 65572 bytes | close `1009` |
| Rate per socket | 500 envelopes/s (burst 1000), 8 MiB/s (burst 16 MiB) | close `1008` "rate limit" |
| Tunnels per instance | 512 | `503 overloaded` |
| Outbound queue per socket | 512 envelopes, plus what the rings held for it at attach | the slow recipient is closed with `1013`; its queue goes back to the ring |
| Heartbeat | a ping every 20 s; silence for 60 s | close `4008` |
| Session lifetime | 50 min, or the desktop JWT's expiry if sooner | close `4000` "reconnect" |
| Revocation (phones) | at once through `DELETE /devices/{id}`, within 2 s by any other path | close `4003`; the device's rings and what was held for it are dropped |

Other close codes:

- `1003`: a text frame.
- `1008` "malformed envelope": an envelope with no ciphertext.
- `1012` "reconnect": the instance is shutting down.

`4000` and `1012` are reconnect hints: dial again at once. The 50-minute close keeps every tunnel under Envoy's
3600 s route timeout.

The limits live in `RelayLimits`; `RelayLimits::validate` refuses inconsistent ones.

## Shutdown and drain

Each tunnel runs on core's `BackgroundTasks` (`spawn_cancellable`). Graceful shutdown cancels its token, and the
tunnel closes with `1012` "reconnect" and is waited for within the background grace. The tunnel also watches core's
`StreamDrain`, so when the connection drain runs out it closes the same way.

## Core's middleware and `server.max_in_flight`

The relay runs inside core's global middleware stack. The integration suite builds that stack in core's order:

- `load_shed`;
- the body limit;
- trailing slash, trace and served-by;
- caller purpose and content negotiation;
- security headers.

It then proves that the upgrade, a round trip and the refusals work through it.

Core's `load_shed` holds its permit only until the handler returns its response. For a WebSocket that is the `101`,
so **an open tunnel holds no `max_in_flight` slot**. The suite proves it with `max_in_flight = 1`: two tunnels open
and a request is still served, while one in-flight plain request does take the only permit. The relay bounds tunnels
itself, with the tunnel limit above.

## Tables

The schema is in `schema/01_dynwork_relay.sql` and `schema/02_dynwork_mobile_auth.sql`. Migrations, when one is needed, go in
`schema/migrations/NNN_<name>.sql`.

- `dynwork_relay_devices`: id, user, kind (`desktop`/`phone`), peer id, name, the credential's SHA-256 (phones
  only), created, last seen, revoked. A revoked row stays for the audit trail. A partial unique index lets a peer
  pair again after a revocation.
- `dynwork_relay_pairings`: id, user, the token's SHA-256 (unique), created, expires, claimed, the device it
  created. Rows that expired a day ago are deleted by the revocation watch, which also drops rings whose envelopes
  have all aged out.
- `dynwork_relay_mobile_grants`: id, user, the consent request token's SHA-256, the code's SHA-256 once allowed, the
  PKCE challenge, `state`, device name, created, expires, decided, approved, redeemed, the device it created. Purged a
  day after expiry like pairings.

All three tables cascade from `users`. The relay stores peer ids, hashes and timestamps: no keys, no ciphertext, no
session content.

## Audit

`connect`, `disconnect` (with its reason and envelope counts), `pair`, `revoke`, `pairing_created`, and for the
mobile sign-in `mobile_consent` (`approved` or `denied`) and `mobile_sign_in`. They go to the
`dynwork_relay::audit` tracing target and carry ids only.

## Single replica

Routing, the ring buffers and presence live in the process. The relay supports **one replica only**:

- Two replicas would split a user's desktop and phone across processes that cannot see each other.
- A Recreate deploy cuts every tunnel; the ends reconnect on `1012`/`4000` and resume end to end.
- Running more replicas needs a shared routing plane (sticky routing by user, or a broker), which this crate does
  not have.

## Tests

The tests live in the `tests/` workspace, as everywhere in this repo:

- `tests/unit/dynwork-relay`: the envelope, the ids, credentials, the rate limiter, the ring buffer, the hub, the
  limits and pairing tokens;
- `tests/integration/dynwork-relay`: real sockets over loopback, core's middleware stack, and the Postgres store on
  a throwaway database.
