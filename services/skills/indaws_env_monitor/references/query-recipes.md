# Recetas de consulta

Lecturas `search_read` listas para usar (solo lectura), expresadas en forma lógica: `search_read(model, domain,
fields, limit, order)`. Ejecútalas con la herramienta de lectura Odoo que tengas disponible. Recuerda: pide **solo**
campos del data dictionary; un campo inexistente rompe la llamada entera. El parque son **68 registros** (67
`connected` + 1 `pending`) → `limit: 100` los cubre.

## Foto general del parque

```
search_read(
  model="database.env",
  domain=[["state","=","connected"]],
  fields=["display_name","odoo_version","odoo_edition","license_expiration_date",
          "last_heartbeat","last_sync_date","last_sync_status","module_count","user_count","url"],
  limit=100, order="display_name"
)
```
Trae todo el parque de una vez y agrega en memoria (versiones, ediciones, hosting, tamaños).

## Churn / inactividad

```
search_read(model="database.env",
  domain=[["state","=","connected"]],
  fields=["display_name","last_activity_date","last_heartbeat"], limit=100,
  order="last_activity_date asc")
```
Calcula `inactivity_days = hoy − last_activity_date` y clasifica con los umbrales 30/60/90 (ver
`metrics-and-thresholds.md`). `last_activity_date = false` = entorno aún no auditado por la sonda → "sin dato", no
dormido. Para explicar el porqué de un entorno concreto, lee su desglose en `env_data['activity']` (ver
`churn-signal.md`).

## Licencias Enterprise que caducan

```
search_read(model="database.env",
  domain=[["odoo_edition","=","enterprise"]],
  fields=["display_name","license_expiration_date","odoo_version"],
  limit=100, order="license_expiration_date asc")
```
Filtra en memoria: caducadas (`< hoy`) y en riesgo (`≤ hoy+60d`).

## Versión / edición (distribución, candidatos a migración)

```
search_read(model="database.env",
  domain=[["state","=","connected"]],
  fields=["display_name","odoo_version","odoo_edition"], limit=100)
```
Agrupa por serie de versión; los de serie 18 (o menor) son candidatos a v19.

## Inventario de módulos de un entorno

Primero localiza el `id` del entorno (por `display_name`/`url`/`database`), luego:
```
search_read(model="database.env.module.version",
  domain=[["database_env_ids","in",[<env_id>]]],
  fields=["technical_name","name","version","author","category_id"],
  limit=500, order="category_id,technical_name")
```

## Cores inDAWS de un entorno

```
search_read(model="database.env.module.version",
  domain=[["database_env_ids","in",[<env_id>]],
          ["category_id","in",[1,2,3,4]]],
  fields=["technical_name","name","version","category_id"], limit=500)
```
(Categorías 1 inDAWS Core, 2 Core Enterprise, 3 Connectors, 4 Inbuilt.)

## Módulos / cores más extendidos del parque

`env_count` de `database.env.module.version` es **computado (no almacenado)** → no vale en `order=`. Usa
`database.env.module` (su `env_count` SÍ está almacenado):
```
search_read(model="database.env.module",
  domain=[["category_id","in",[1,2,3,4]]],
  fields=["technical_name","name","category_id","env_count"],
  limit=500, order="env_count desc")
```
Para el catálogo de categorías y sus `module_count`:
```
search_read(model="database.env.module.category",
  fields=["name","path_pattern","module_count"], limit=20, order="sequence")
```

## Módulos custom más grandes del parque (líneas de código)

`loc_total` vive en `database.env.module.version`, una fila por `(technical_name, version)` — no es acumulable por
entorno como `env_count`, es el tamaño de ESA versión del módulo (compartido por todos los que la tienen instalada).
```
search_read(model="database.env.module.version",
  domain=[["loc_total",">",0]],
  fields=["technical_name","version","loc_total","category_id"],
  limit=100, order="loc_total desc")
```
Un mismo `technical_name` puede aparecer varias veces (una por versión distinta en el parque) con LOC distinto entre
versiones. Ignora los que salen con `loc_total = 0`/`false`: son Community/Enterprise oficial (Cloc los excluye
a propósito) o módulos custom aún no auditados por esta acción.

## Footprint de código (LOC) de un entorno concreto

Primero localiza el `id` del entorno, luego:
```
search_read(model="database.env.module.version",
  domain=[["database_env_ids","in",[<env_id>]],["loc_total",">",0]],
  fields=["technical_name","version","loc_total","category_id"],
  limit=500, order="loc_total desc")
```
Suma `loc_total` para el total de código custom de ese cliente; el desglose por fila te dice qué módulo pesa más. Si
salen 0 filas, no necesariamente el cliente no tiene personalización: puede que sus módulos custom aún no tengan
`loc_total` calculado (flota en proceso de rellenarse) — compruébalo contra su inventario completo de módulos antes
de concluir "sin código custom".

## Footprint de personalización de un entorno (complejidad de migración)

```
search_read(model="database.env",
  domain=[["id","=",<env_id>]], fields=["display_name","env_data"], limit=1)
```
De `env_data` lee los `count` de `studio_fields`, `studio_views`, `automated_actions`, `custom_crons` y el
`security_audit`. Reporta volúmenes, no horas.

## Servidores caídos / liveness

```
search_read(model="database.env",
  domain=[["state","=","connected"]],
  fields=["display_name","last_heartbeat","last_sync_date","last_sync_status"],
  limit=100, order="last_heartbeat asc")
```
`last_heartbeat` muy antiguo → servidor probablemente caído (es liveness, no uso humano).

## Identidad del solicitante (opcional)

Si necesitas personalizar la respuesta para quien pregunta, consulta su identidad con la herramienta de identidad
Odoo disponible (`whoami`).
