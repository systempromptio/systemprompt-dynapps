# Discuss / Bus — Integrations

Cómo el bus (`bus.bus` + websocket) se integra con el resto del ecosistema. A diferencia de `mail`, que se HEREDA por mixins, el bus se CONSUME: el código de aplicación emite notificaciones por `bus.listener.mixin._bus_send` / `bus.bus._sendone`, y el cliente las recibe vía un único websocket (o longpolling). Ubicación: `odoo18/addons/bus/` y `odoo18/addons/mail/models/discuss/`.

Para el core del bus ver `core.md`; para gotchas ver `gotchas.md` (BUSG1-BUSG31); para v19 ver `v19_changes.md`.

---

## 1. Cadena end-to-end (server → browser)

```
[CÓDIGO DE APLICACIÓN]
   record._bus_send("type", payload, subchannel=None)        bus_listener_mixin.py:15-22
        │   for record in self: main_channel = record._bus_channel()  (default: self, :24-26)
        │   main_channel.ensure_one(); assert isinstance(.., models.Model)   :19-20
        ▼
   bus.bus._sendone(channel, type, message)                  bus.py:105-128
        │   channel_with_db(db, target)  → (db, model, id[, sub])         bus.py:57-64
        │   cr.precommit.data["bus.bus.values"].append({channel, message}) bus.py:117-127
        │   cr.postcommit.data["bus.bus.channels"].add(channel)            bus.py:128
        ▼
   ── COMMIT de la transacción de negocio ──
        │
   precommit hook create_bus()  → INSERT bulk en bus_bus     bus.py:134-136
        ▼  (filas persistidas ANTES del commit)
   ── COMMIT efectivo ──
        ▼
   postcommit hook notify()                                  bus.py:145-163
        │   get_notify_payloads(channels)  (split >8000B)     bus.py:67-84
        │   cursor a BD "postgres": SELECT pg_notify('imbus', payload)     bus.py:155-163
        ▼
[POSTGRESQL]  canal LISTEN/NOTIFY  'imbus'
        ▼
   ImDispatch.loop()  (1 thread por worker)                  bus.py:233-254
        │   cr.execute("listen imbus"); selector EVENT_READ   bus.py:236-241
        │   conn.poll(); parsea payload JSON → lista de canales            bus.py:245-247
        │   for ws in self._channels_to_ws[channel]: ws.trigger_notification_dispatching()  bus.py:251-254
        ▼
[WEBSOCKET]  (gevent worker)
        │   trigger_notification_dispatching → socketpair.send(b'x')  websocket.py:366-382
        │   get_messages() detecta notif socket readable      websocket.py:323-324
        │   _dispatch_bus_notifications()                     websocket.py:688-731
        │       env["bus.bus"]._poll(channels, last, ignore)  bus.py:166-185
        │       dedup vía _notif_history / MAX_NOTIFICATION_HISTORY_SEC   websocket.py:707-730
        │       self._send(notifications)  → frame TEXT       websocket.py:731
        ▼
[BROWSER]  websocket_worker.js (SharedWorker o Worker)
        │   onmessage → broadcast("notification") a todas las tabs        workers/websocket_worker.js
        ▼
   bus_service (OWL) → notificación a los servicios suscritos
```

Punto clave de timing: la notif se INSERTA en precommit y el `pg_notify` se dispara en postcommit (`bus/models/bus.py:141-144`, `ImBus._ensure_hooks`, lo documenta). Si se hiciera el NOTIFY antes del commit, el `_poll` del websocket no encontraría la fila → notificación perdida.

---

## 2. Escenario multi-worker (`bus/models/bus.py:202-271`, `ImDispatch`)

El bus está diseñado para N workers concurrentes (gevent/gunicorn) compartiendo una BD:

- Cada proceso worker tiene **su propio `ImDispatch`** (singleton de módulo `dispatch = ImDispatch()`, `bus/models/bus.py:269`, `dispatch`) que hace `LISTEN imbus` sobre la BD `postgres` (`bus/models/bus.py:236-238`, `ImDispatch.loop`).
- PostgreSQL entrega el `NOTIFY 'imbus'` a **todos** los listeners → cada worker recibe la lista de canales y la cruza con SU `_channels_to_ws` local (`bus/models/bus.py:251-252`, `ImDispatch.loop`).
- Sólo despachan los workers que tienen websockets suscritos a esos canales; el resto ignora.
- El websocket vive en el **gevent worker** (puerto `gevent_port`, `bus/websocket.py:943`, `WebsocketConnectionHandler.open_connection`). El handshake exige que la conexión entre por ese puerto o falla con `RuntimeError`.
- `ImDispatch` tiene **lazy start** (`bus/models/bus.py:219-221`, `ImDispatch.subscribe`): el thread arranca en el primer `subscribe`. Tras `CommonServer.on_stop` se setea `stop_event` (`bus/models/bus.py:271`).
- **Implicación**: con multi-worker, el `NOTIFY` cruza el límite de proceso vía PostgreSQL — no hay estado compartido en memoria entre workers. Si un worker pierde su `LISTEN` (reconexión de cursor), pierde notifs durante la caída (gotcha BUSG5).

---

## 3. bus ↔ discuss.channel (`mail/models/discuss/`)

`discuss.channel` hereda `["mail.thread", "bus.listener.mixin"]` (`mail/models/discuss/discuss_channel.py:32`, `Channel._inherit`) — es el mayor consumidor del bus.

- **Resolución de canales por membresía**: `ir.websocket._build_bus_channel_list` override en discuss (`mail/models/discuss/ir_websocket.py:63-84`, `IrWebsocket._build_bus_channel_list`) busca los `discuss.channel` donde `is_member=True` (o IDs explícitos del cliente, p.ej. livechat público) y añade:
  - el propio canal `discuss.channel` (modelo) — broadcast a todos los miembros;
  - el subchannel `(channel, "members")` (`bus/models/ir_websocket.py:82`, `IrWebsocket._prepare_subscribe_data`) — sólo a miembros activos. **v19 renombra a `(channel, "internal_users")`** (ver `v19_changes.md` DISC-C05).
- **Mensaje nuevo**: `discuss.channel._notify_thread` (`mail/models/discuss/discuss_channel.py:649-660`, `Channel._notify_thread`) emite `discuss.channel/new_message` al canal y `is_pinned` al subchannel `members`.
- **Eventos de canal**: delete (`mail/models/discuss/discuss_channel.py:307`, `Channel._unlink_except_all_employee_channel`), join (`mail/models/discuss/discuss_channel.py:460`, `Channel._add_members`), leave (`mail/models/discuss/discuss_channel.py:382,392`, `Channel._action_unfollow`), unpin (`mail/models/discuss/discuss_channel.py:1098`, `Channel.channel_pin`) usan `_bus_send` / `_bus_send_store`.
- `_bus_send_store` lo define la versión mail del mixin (`mail/models/bus_listener_mixin.py:10`, `BusListenerMixin._bus_send_store`), que serializa con `Store` y respeta el mismo flujo precommit/postcommit.
- **Typing / reactions / RTC / seen**: todos emiten por bus inmediatamente, sin rate-limit servidor (gotchas BUSG15-BUSG17).

---

## 4. bus ↔ mail.thread (notificaciones de chatter)

- `mail.thread._notify_thread` y derivados emiten al partner destinatario vía `_bus_send_store` (`mail/models/mail_thread.py:3275`, `MailThread._notify_thread_by_inbox`, `4709`).
- El canal destino es el `res.partner` (modelo). El partner se autosuscribe en `_build_bus_channel_list` (`bus/models/ir_websocket.py:69-70`, `IrWebsocket._build_bus_channel_list`), de modo que cualquier `partner._bus_send(...)` llega a todas las pestañas del usuario logueado.
- Web push (fallback offline): `_notify_thread_by_web_push` (`mail/models/discuss/discuss_channel.py:683`, `Channel._notify_thread_by_web_push`) usa un canal aparte (no bus) para móviles con la app cerrada.

---

## 5. bus ↔ presence (im_status) — v18

- `bus.presence._send_presence` (`bus/models/bus_presence.py:99-117`, `BusPresence._update_presence`) emite `bus.bus/im_status_updated` al canal `(partner, "presence")`.
- Suscripción: el cliente manda canales `odoo-presence-res.partner_<id>`; el servidor los traduce en `_build_presence_channel_list` (`bus/models/ir_websocket.py:38-56`, `IrWebsocket._build_presence_channel_list`) validando visibilidad (sólo internos o partners con canal compartido — discuss extiende en `mail/models/discuss/ir_websocket.py:29-60`, `IrWebsocket._build_presence_channel_list`).
- Update por poll: `_update_bus_presence` (`bus/models/ir_websocket.py:125-131`, `IrWebsocket._update_bus_presence`) llama `bus.presence.update_presence` que hace `cr.commit()` propio.
- Cierre: `_on_websocket_closed` (`bus/models/ir_websocket.py:133-135`, `IrWebsocket._on_websocket_closed`) marca `status='offline'`.
- **v19**: TODO esto se elimina del módulo bus (BUS-C01/C04/C05). La presencia pasa a gestionarse en mail/discuss. No asumir que `partner.im_status` lo provee el bus en v19.

---

## 6. Longpolling como fallback (`bus/controllers/`)

- Cuando no hay websocket (Safari sin SharedWorker, proxies que cortan upgrades, navegadores antiguos), el worker degrada a `/websocket/peek_notifications` (`bus/controllers/websocket.py:31-43`, `WebsocketController.peek_notifications`).
- Misma resolución de canales (`_prepare_subscribe_data`) y mismo `_poll`, pero SIN dispatch en tiempo real: el cliente hace polling.
- La presencia es una llamada separada `/websocket/update_bus_presence` (`bus/controllers/websocket.py:45-50`, `WebsocketController.update_bus_presence`) — si el cliente sólo hace `peek_notifications`, queda "offline" tras 65s (gotcha BUSG12).
- `/bus/has_missed_notifications` (`bus/controllers/main.py:15-19`, `BusController.has_missed_notifications`): el cliente detecta tras reconectar si su `last_notification_id` ya fue GC'd (si la fila no existe → hubo gap).

---

## 7. Diagnóstico rápido

| Síntoma | Probable causa | Dónde mirar |
|---------|----------------|-------------|
| Notif no llega tras `_sendone` sin commit | precommit defiere INSERT | `bus/models/bus.py:117-136`, `ImBus._sendone`, gotcha BUSG1 |
| "imbus payload too large" en logs | >8000B, split | `bus/models/bus.py:150-154`, `_ensure_hooks.notify`, BUSG2 |
| Notif perdida en transacción larga | fuera de ventana 10s | `bus/websocket.py:259`, `Websocket.MAX_NOTIFICATION_HISTORY_SEC`, BUSG3 |
| Cliente reconecta cada poco | worker version != `_VERSION` | `bus/websocket.py:908`, `WebsocketConnectionHandler._VERSION` y `bus/websocket.py:1034`, `WebsocketConnectionHandler._serve_forever`, BUSG8 |
| Usuario "offline" pero conectado | longpolling sin update_presence | `bus/controllers/websocket.py:45`, `WebsocketController.update_bus_presence`, BUSG12 |
| Subscribe pierde canales previos | subscribe REEMPLAZA | `bus/models/bus.py:207-221`, `ImDispatch.subscribe`, `bus/websocket.py:356-364`, `Websocket.subscribe`, BUSG6 |
