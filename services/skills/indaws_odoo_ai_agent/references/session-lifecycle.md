# Ciclo de vida de sesión — heartbeat, reaper y accounting (anti-fuga)

> Patrón del build **web_shell** (2026-07). Conocimiento CARO: la fuga de sesiones
> "activas" nos mordió dos veces. El síntoma: "Concurrent session limit reached (N)"
> permanente aunque no haya nadie usando el agente.

## Por qué se fugan
Una fila de sesión queda `state='active'` para siempre cuando el contenedor/WS muere
SIN cierre limpio (SIGKILL, host reboot, rebuild de imagen, F5 que no propaga el
close). Sin reaper, esas zombis llenan la cuota concurrente **para siempre**.

## Heartbeat POR SESIÓN (no por usuario) — el error clásico
Tentación: que el reporter de uso (per-contenedor/per-uid) refresque `last_seen`.
**MAL** → bumpea TODAS las sesiones del uid, incluidas las zombis → nunca caducan.
Se detecta porque las zombis comparten `last_seen` idéntico.

Correcto: **el propio sidecar de esa sesión** hace ping de liveness (hilo daemon
~60s, y el bucle PTY cada ~60s) a un endpoint que bumpea **solo esa** fila
(`_touch_last_seen(session_id)`). Una zombi (sidecar muerto) no late → caduca.

## Reaper
Cierra `active` cuyo `last_seen` supere una ventana (`session_stale_min`, def ~10min
≫ el latido de 60s; o `create_date` si nunca hubo latido). Llámalo en el self-heal al
abrir (`_check_open`) y en `@api.autovacuum`. Al cerrar, libera lease + runtime + revoca bearers.

## Accounting justo (no wall-clock)
El tiempo de shell/uso NO se cuenta `create_date → closed_at` (una zombi colgada 90
min contaría 90 min muertos). Cuenta **tiempo VIVO** = `create_date → last_seen`
(el heartbeat marca el último instante conocido de vida). Una zombi solo cuenta lo
que estuvo viva.

## Cierre normal
La ruta limpia (idle-close 4001, cierre de pestaña, drop) pasa por un `finally` que
avisa a Odoo (`_notify_closed`) → marca `closed`. El reaper es solo para los muertos
sin cierre limpio.
