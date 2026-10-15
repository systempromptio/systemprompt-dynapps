# Señal de actividad y churn

## Qué mide `last_activity_date`

`last_activity_date` = **última actividad HUMANA real** del entorno, que calcula la sonda de actividad (acción
*Activity probe*) como `max(last_presence, last_comment)`. Es la base del churn:
`inactivity_days = hoy − last_activity_date`, con umbrales 30/60/90 (ver `metrics-and-thresholds.md`).

El desglose por entorno está en `env_data['activity']`:

| Señal | Tier | Por qué |
|---|---|---|
| `last_presence` | **ORO** (mueve la fecha) | sesión de navegador real; no avanza sin un humano dentro |
| `last_comment` | **ORO** (mueve la fecha) | nota tecleada en el chatter (excluye OdooBot); no avanza sin un humano |
| `last_login` | contexto | un login (también los hace la API/integraciones) → no mueve la fecha |
| `last_email` | contexto | correos, casi siempre automáticos → no mueve la fecha |
| `last_backend_write` | contexto | escritura en documento de negocio (crons/API también) → no mueve la fecha |
| `busiest_model` | contexto | modelo con la escritura más reciente |

Solo las dos señales ORO mueven `last_activity_date`. Por eso un entorno donde solo corren crons/API/integraciones
(sin que nadie entre ni escriba notas) cuenta como **sin uso humano** = churn — que es justo lo que se quiere detectar.

## Matices al interpretar

- **`last_presence` es efímero:** Odoo (v18 `bus.presence` / v19 `mail.presence`) purga la fila ~12 h tras el último
  poll, y solo se crea con una sesión de navegador. Para un entorno dormido casi siempre vendrá vacío → la fecha ORO
  la sostiene `last_comment`. Es por diseño: presence CONFIRMA "activo ahora", no DATA la última actividad de un
  dormido.
- **`last_activity_date = false`**: el entorno aún no ha sido auditado por la sonda → trátalo como **sin dato**, no
  como dormido. No concluyas churn sin la señal.
- **Liveness ≠ uso humano:** `last_heartbeat` / `last_sync_date` y `login_date` (que además suele venir vacío) miden
  que el servidor/agente responde, no que un humano use el sistema. No los presentes como "actividad".
