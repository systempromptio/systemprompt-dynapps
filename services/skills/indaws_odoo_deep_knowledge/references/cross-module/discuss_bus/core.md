# Discuss / Bus — Odoo 18.0

<!-- verified: v18.0 @ 2026-06 -->

## 1. Architecture Overview

```
bus.bus (PostgreSQL message queue)
  ├── channel: JSON tuple (dbname, model_name, id, [subchannel])
  ├── message: JSON {"type": notification_type, "payload": data}
  └── GC: 24h retention (bus.gc_retention_seconds)

Message Flow:
  Application → _sendone() → precommit hook (defer) → DB INSERT
  → postcommit hook → PostgreSQL NOTIFY 'imbus'
  → ImDispatch (LISTEN) → Websocket → Browser

bus.presence (User/Guest online status)
  ├── user_id (bus base) — guest_id lo AÑADE mail (mail/.../bus_presence.py:9)
  ├── índice único parcial sobre user_id (bus) y sobre guest_id (mail)
  ├── XOR user_id/guest_id (constraint definido en mail, bus_presence.py:15)
  ├── last_poll, last_presence (timestamps), _log_access = False
  └── status: online | away (>30min inactivo, sigue polleando) | offline (sin poll 65s)

discuss.channel (Conversation)
  ├── channel_type: chat | channel | group
  ├── channel_member_ids (O2M discuss.channel.member)
  ├── group_ids (M2M — auto-subscription)
  ├── uuid (invitation link)
  ├── sfu_channel_uuid, sfu_server_url (RTC/video)
  └── parent_channel_id (threaded sub-channels)

mail.guest (Anonymous access)
  ├── access_token (UUID)
  ├── channel_ids (M2M)
  └── Cookie: dgid = {guest_id}|{access_token}
```

## 2. Bus Model (bus.bus)

### Channel Formats (`channel_with_db`, `bus/models/bus.py:57-64`, `def channel_with_db`)
```python
# String channel → (dbname, "broadcast")
"broadcast"

# Model instance → (dbname, model._name, model.id)
record  # ej. partner → (db, "res.partner", 123)

# (Model, subchannel) tuple → (dbname, model._name, model.id, subchannel)
(partner, "presence")  # → (db, "res.partner", 123, "presence")

# Cualquier otro tuple → se devuelve tal cual (ya normalizado)
```
- El cliente sólo puede enviar canales STRING en `subscribe` (`bus/models/ir_websocket.py:92-93`, `IrWebsocket._prepare_subscribe_data` valida `isinstance(c, str)`); los canales de modelo los añade el servidor en `_build_bus_channel_list`.

### Notification Lifecycle (`bus/models/bus.py`)
1. `_sendone(target, notification_type, message)` (`bus/models/bus.py:105-128`, `ImBus._sendone`) → `channel_with_db()` (`bus/models/bus.py:57-64`, `channel_with_db`) normaliza target → append a `cr.precommit.data["bus.bus.values"]`. NO inserta inmediatamente.
2. Precommit hook `create_bus()` (`bus/models/bus.py:134-136`, `_ensure_hooks.create_bus`): bulk `create()` (sudo) en `bus_bus` al cierre de transacción, antes de COMMIT.
3. Postcommit hook `notify()` (`bus/models/bus.py:146-163`, `ImBus._ensure_hooks`, donde `notify()` va anidada): tras COMMIT, abre cursor a la BD `postgres` y ejecuta `SELECT pg_notify('imbus', payload)`. El nombre de la función es configurable vía env `ODOO_NOTIFY_FUNCTION` (`bus/models/bus.py:28`, `ODOO_NOTIFY_FUNCTION`).
4. Payload `NOTIFY`: límite `NOTIFY_PAYLOAD_MAX_LENGTH = 8000` bytes (`bus/models/bus.py:31-42`, `get_notify_payload_max_length`, override env `ODOO_NOTIFY_PAYLOAD_MAX_LENGTH`). `get_notify_payloads()` (`bus/models/bus.py:67-84`, `get_notify_payloads`) parte la lista de canales recursivamente por la mitad si excede; loguea INFO si hubo split (`bus/models/bus.py:150-154`, `_ensure_hooks.notify`).
5. Mensaje persistido: `{"type": notification_type, "payload": message}` serializado con `json_dump` separadores compactos (`bus/models/bus.py:48-49`, `def json_dump`, `120-126`).

### Deduplication Window (`bus/websocket.py`)
- `MAX_NOTIFICATION_HISTORY_SEC = 10` (`bus/websocket.py:259`, `Websocket.MAX_NOTIFICATION_HISTORY_SEC`) — ventana en la que se retiene el historial de ids ya despachados por websocket.
- `_last_notif_sent_id` (`bus/websocket.py:284`, `Websocket.__init__`) se mantiene RETRASADO unos segundos respecto al id máximo: las ids se asignan al pedirlas pero las notifs se despachan al commit, así que una id menor puede llegar después de una mayor.
- `_notif_history` (`bus/websocket.py:287`, `Websocket.__init__`) lista ordenada `(notif_id, send_time)`; al despachar (`bus/websocket.py:688-731`, `Websocket._dispatch_bus_notifications`) se descartan duplicados por id y se avanza `_last_notif_sent_id` sólo cuando los ids más bajos superan los 10s.
- **Implicación**: transacciones largas (>10s) entre `create` de la notif y el COMMIT pueden no despacharse nunca si el usuario recibe otra notif entre medias. Crear las notifs lo más cerca posible del commit (`bus/websocket.py:252-258`, `Websocket`).

## 3. Websocket

### Connection Lifecycle (`bus/websocket.py`)
```
HTTP Upgrade (RFC6455 handshake, Sec-WebSocket-Version: 13)
  → _assert_handshake_validity (websocket.py:994-1025)
  → _serve_forever (websocket.py:1027-1057)
  → si version != worker _VERSION → close CLEAN "OUTDATED_VERSION" (websocket.py:1034-1044)
  → Subscribe event (channels + last_notification_id)
  → Active: get_messages() generator (websocket.py:304-330) — selector sobre socket + notif socketpair
  → Close: clean close con code/reason O abnormal (timeout)
```

### Protocolo de handshake
- `SUPPORTED_VERSIONS = {'13'}` (`bus/websocket.py:897`, `WebsocketConnectionHandler.SUPPORTED_VERSIONS`) — versión del protocolo websocket RFC6455, NO la del worker.
- `_VERSION = "18.0-7"` (`bus/websocket.py:908`, `WebsocketConnectionHandler._VERSION`) — versión del bundle del worker JS; se incrementa cada vez que cambia `websocket_worker.js`. Mismatch → cierre CLEAN con razón `"OUTDATED_VERSION"` (no usa close code custom para no romper el backoff exponencial del worker).
- Mismatch de `SUPPORTED_VERSIONS` → `UpgradeRequired` HTTP 426 (`bus/websocket.py:68-78`, `class UpgradeRequired`, `1014-1015`).

### Configuración (constantes reales)
- **Keep-alive**: `KEEP_ALIVE_TIMEOUT = config['websocket_keep_alive_timeout']`, **default 3600s** (`bus/websocket.py:743`, `TimeoutManager.KEEP_ALIVE_TIMEOUT`; default en `odoo/tools/config.py:81`, `configmanager.__init__`). Se le suma jitter aleatorio `[0, KEEP_ALIVE/2]` por conexión para evitar timeouts simultáneos (`bus/websocket.py:759-761`, `TimeoutManager.__init__`).
- **Selector TIMEOUT**: `TimeoutManager.TIMEOUT = 15` (`bus/websocket.py:740`, `TimeoutManager.TIMEOUT`) — máximo que bloquea el selector y timeout de espera de PONG/CLOSE.
- **PING**: se envía si la conexión lleva ociosa `INACTIVITY_TIMEOUT = 40s` (`CONNECTION_TIMEOUT 60 - 20`, `bus/websocket.py:749-750`, `TimeoutManager.CONNECTION_TIMEOUT`). Peor caso de envío = `INACTIVITY_TIMEOUT + TIMEOUT = 55s` (el selector puede bloquear hasta 15s). El "~55s" es el peor caso, no el intervalo.
- **Rate limit**: `RL_BURST = config['websocket_rate_limit_burst']` (**default 10**) y `RL_DELAY = config['websocket_rate_limit_delay']` (**default 0.2s**, NO 0.3) (`bus/websocket.py:261-263`, `class Websocket`; defaults en `odoo/tools/config.py:82-83`, `configmanager.__init__`). `_limit_rate()` (`bus/websocket.py:652-665`, `Websocket._limit_rate`) levanta `RateLimitExceededException` si llegan ≥BURST frames en menos de `RL_DELAY * RL_BURST` segundos → cierre con code TRY_LATER (1013).
- **MESSAGE_MAX_SIZE**: `2 ** 20` = 1 MiB por mensaje (frame único o fragmentado) (`bus/websocket.py:233`, `Websocket.MESSAGE_MAX_SIZE`).

### Close codes (`bus/websocket.py:160-176`, `CloseCode`)
- `1000 CLEAN`, `1001 GOING_AWAY` (server shutdown `_kick_all`, `bus/websocket.py:1060-1067`, `def _kick_all`), `1006 ABNORMAL_CLOSURE`, `1009 MESSAGE_TOO_BIG`, `1011 SERVER_ERROR`, `1013 TRY_LATER` (PoolError/RateLimit).
- Custom Odoo: `4001 SESSION_EXPIRED`, `4002 KEEP_ALIVE_TIMEOUT`.

### vs Longpolling (Fallback)
- Route: `/websocket/peek_notifications` (`type='json'`, `auth='public'`, `cors='*'`) — `bus/controllers/websocket.py:31-43`, `WebsocketController.peek_notifications`.
- Usado cuando el websocket no está disponible (CORS, proxy, navegadores viejos, Safari sin SharedWorker).
- Primer poll (`last=0`): devuelve el buffer de los últimos `TIMEOUT = 50s` (`bus/models/bus.py:24`, `TIMEOUT`; `_poll` en `bus/models/bus.py:166-185`, `ImBus._poll`).
- Polls posteriores (`last>N`): notifs con `id > N` filtradas por canal (`bus/models/bus.py:171-177`, `ImBus._poll`).
- NO actualiza presencia: `update_bus_presence` es ruta separada (`bus/controllers/websocket.py:45-50`, `WebsocketController.update_bus_presence`).

## 4. Presence System

### Timers (`bus/models/bus_presence.py:8-11`, `UPDATE_PRESENCE_DELAY` — v18; **eliminado en v19**)
| Constant | Value | Purpose |
|----------|-------|---------|
| UPDATE_PRESENCE_DELAY | 60s | Client poll frequency (`bus/models/bus_presence.py:8`, `UPDATE_PRESENCE_DELAY`) |
| DISCONNECTION_TIMER | 65s (`UPDATE_PRESENCE_DELAY + 5`) | No poll → offline (`bus/models/bus_presence.py:9`, `DISCONNECTION_TIMER`) |
| AWAY_TIMER | 1800s (30min) | Inactivity → away (`bus/models/bus_presence.py:10`, `AWAY_TIMER`) |
| PRESENCE_OUTDATED_TIMER | 12h | GC stale presences (`bus/models/bus_presence.py:11`, `PRESENCE_OUTDATED_TIMER`; `bus/models/bus_presence.py:139-142`, `BusPresence._gc_bus_presence`) |

> El modelo `bus.presence` se ELIMINA en v19 (ver `v19_changes.md` BUS-C01). En v18 es `_log_access = False` con índice único parcial sobre `user_id` (`bus/models/bus_presence.py:23`, `class BusPresence`; el índice se crea en `bus/models/bus_presence.py:30-31`, `BusPresence.init`).

### Status Computation
- **online**: `inactivity_period <= AWAY_TIMER*1000` en el último poll (`bus/models/bus_presence.py:93-104`, `BusPresence._update_presence`).
- **away**: `inactivity_period > AWAY_TIMER` (30min) pero sigue polleando.
- **offline**: sin poll durante `DISCONNECTION_TIMER=65s`, o explícito al cerrar (`_on_websocket_closed`, `bus/models/ir_websocket.py:135`, `IrWebsocket._on_websocket_closed`).
- `im_status` (campo en `res.partner`/`res.users`/`mail.guest`) NO está almacenado: `compute` sin store, recomputado por recordset con UNA query a `bus.presence` (`bus/models/res_partner.py:12-23`, `ResPartner._compute_im_status`; `bus/models/res_users.py:12-19`, `ResUsers._compute_im_status`; `mail/models/discuss/mail_guest.py:63-68`, `MailGuest._compute_im_status`), invalidado por `_invalidate_im_status` (`bus/models/bus_presence.py:106-108`, `BusPresence._invalidate_im_status`). Partner sin usuario → `"im_partner"`; con usuario sin presence → `"offline"`. Partner multi-usuario colapsa con preferencia "online" (`bus/models/res_partner.py:14-20`, `ResPartner._compute_im_status`).

### Broadcasting (`bus/models/bus_presence.py:99-117`, `BusPresence._update_presence`)
- `_send_presence()` → `_sendone((partner, "presence"), "bus.bus/im_status_updated", {...})` con `presence_status`, `im_status`, `partner_id` (`bus/models/bus_presence.py:108-117`, `BusPresence._invalidate_im_status`).
- Sólo dispara en `create()`/`write()` cuando el `status` cambia (`bus/models/bus_presence.py:33-38`, `BusPresence.create`; `bus/models/bus_presence.py:39-45`, `BusPresence.write`).
- `update_presence()` (`bus/models/bus_presence.py:63-78`, `BusPresence.update_presence`) hace `cr.commit()` por poll y silencia errores de concurrencia con `mute_logger('odoo.sql_db')` + `PG_CONCURRENCY_EXCEPTIONS_TO_RETRY` → multi-tab no rompe pero puede perder el write.

## 5. Discuss Channels

### Types
| Type | Members | Searchable | Auto-subscribe |
|------|---------|-----------|----------------|
| chat | Max 2 | No | No |
| channel | Unlimited | Yes | By group_ids |
| group | Unlimited | No (invitation) | No |

### Real-Time Features
- **Typing**: client-side debounce → broadcast to (channel, "members"), no server storage
- **Seen status**: per-member seen_message_id, broadcast on update
- **Reactions**: mail.message.reaction model, immediate broadcast, no rate limit
- **RTC**: discuss.channel.rtc.session per member, SFU server for media
- **Threads**: parent_channel_id sub-channels, auto-GC if unpinned + stale

## 6. Guest System (mail.guest)

- Definido en mail, NO en bus: `mail/models/discuss/mail_guest.py`. Hereda `bus.listener.mixin` (`mail/models/discuss/mail_guest.py:45`, `MailGuest._description`).
- Website visitors join livechat/channels via invitation
- Cookie `dgid` = `{id}|{access_token}` (`_cookie_name='dgid'`, `_cookie_separator='|'` en `mail/models/discuss/mail_guest.py:48-49`, `class MailGuest`); `_format_auth_cookie` = `f"{id}|{access_token}"` (`mail/models/discuss/mail_guest.py:144-151`, `MailGuest._format_auth_cookie`).
- `_get_guest_from_token` (`mail/models/discuss/mail_guest.py:70-80`, `MailGuest._get_guest_from_token`): split por `|`, `browse(id).sudo()`, `consteq(access_token, ...)` (tiempo constante sobre el token). El `id` no es secreto; el token sí.
- Presencia: `guest_id` se añade a `bus.presence` desde mail (`mail/models/bus_presence.py:9`, `BusPresence.guest_id`), no existe en el módulo bus.
- No logout mechanism — stays "online" until 12h GC (o `_on_websocket_closed` por cookie, `mail/models/discuss/ir_websocket.py:100-107`, `IrWebsocket._on_websocket_closed`)
- Channels validated against guest token on subscription (`_build_bus_channel_list`, `mail/models/discuss/ir_websocket.py:63-84`, `IrWebsocket._build_bus_channel_list`)

## 7. Security

### Channel Authorization
- `has_access()` on discuss.channel
- Write: is_member + owner/moderator
- Read: is_member + not archived
- Group-based: group_public_id controls access

### Bus Subscription Safety
- `_prepare_subscribe_data()` (`bus/models/ir_websocket.py:73-117`, `IrWebsocket._prepare_subscribe_data`) valida que los canales del cliente sean strings y resetea `last` a 0 si supera `_bus_last_id()`.
- `_build_bus_channel_list()` (`bus/models/ir_websocket.py:58-71`, `IrWebsocket._build_bus_channel_list`) añade SIEMPRE `'broadcast'`, los `groups_id` del usuario y el `partner` si autenticado. El cliente NO elige canales de modelo arbitrarios: discuss los resuelve por membresía (`mail/models/discuss/ir_websocket.py:63-84`, `IrWebsocket._build_bus_channel_list`).
- Subchannel `(channel, "members")` sólo para canales donde el usuario es miembro (`mail/models/discuss/ir_websocket.py:82`, `IrWebsocket._build_bus_channel_list`).
- NO hay autorización por notificación en `bus.bus`: la seguridad se asume en el emisor (`_sendone` docstring `bus/models/bus.py:106-114`, `ImBus._sendone` advierte que `target` string "no debe ser adivinable por un atacante"). El cliente sólo recibe canales a los que el servidor le suscribió.

### Guest (`mail/models/discuss/mail_guest.py`)
- Cookie `dgid` = `{id}|{access_token}` (`_cookie_name = 'dgid'`, `mail/models/discuss/mail_guest.py:48`, `MailGuest._cookie_name`).
- `_get_guest_from_token()` (`mail/models/discuss/mail_guest.py:70-80`, `MailGuest._get_guest_from_token`) compara el token con `consteq()` (tiempo constante, `mail/models/discuss/mail_guest.py:78`, `MailGuest._get_guest_from_token`) para evitar timing attacks.
