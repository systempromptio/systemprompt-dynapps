---
name: indaws-dynapps-db-upgrade-runner
description: >
  Executes the Odoo database upgrade via the official Upgrade Service (upgrade.odoo.com)
  in a local on-premise environment. Submits the production dump, downloads the upgraded
  result, and restores it locally. Use when the user says "upgrade the database", "run the
  upgrade", "submit DB to Upgrade Service", or when odoo-database-upgrader asks for an
  upgraded base dump.
version: 1.2.2
model: sonnet
effort: medium
allowed-tools: [Bash, Read, Write, AskUserQuestion]
---

# Dynapps DB Upgrade Runner

Prepara la base de datos migrada (`${CUSTOMER}_upgraded_base`) que `odoo-database-upgrader`
necesita como punto de partida. Entorno: **on-premise local**. Sin credenciales adicionales —
el Upgrade Service lee la suscripción enterprise directamente de la BD.

## Workflow

### 1. Pre-upgrade

- Confirmar versión destino (`VERSION_DESTINO`) y nombre del cliente (`CUSTOMER`).
- Verificar que PostgreSQL está activo y la BD origen existe.
- Recordar: Odoo debe estar detenido antes de enviar la BD (evita emails durante el upgrade).

### 2. Ejecutar upgrade on-premise

Ver `references/path-on-premise.md` — comandos exactos.

Flujo resumido:
```bash
# Dump de la BD origen
pg_dump -Fc -U odoo ${CUSTOMER}_v${VERSION_ORIGEN} > /tmp/${CUSTOMER}_prod.dump

# Enviar al Upgrade Service (lee suscripción de la propia BD)
python3 <(curl -s https://upgrade.odoo.com/upgrade) test \
  -d /tmp/${CUSTOMER}_prod.dump \
  -t ${VERSION_DESTINO}.0 \
  -x \
  --output /tmp/${CUSTOMER}_upgraded.dump
```

El script descarga el dump resultado cuando el upgrade termina en los servidores de Odoo.
Puede tardar desde minutos hasta horas según el tamaño de la BD.

### 3. Post-restauración

Ver `references/post-restore.md` — pasos comunes:
- Restaurar dump en local
- **Neutralizar la BD (GATE OBLIGATORIO)** — ver `references/neutralization.md`
- Crear copia dev

> ⚠️ **El dump es de producción y llega SIN neutralizar.** La neutralización NO es
> opcional ni "automática por versión": se ejecuta (`odoo-bin neutralize` +
> `neutralize_extras.sql`) y se **verifica** con un gate duro **antes de arrancar
> cualquier `odoo-bin`**. Detalle completo en `references/neutralization.md`.
>
> **Después de neutralizar bien queda UN servidor de correo activo, y tiene que quedarse.** Es el
> falso que inserta Odoo (`host = invalid`) para que la base no caiga al `smtp_server` del
> `odoo.cfg`: sin ningún `ir.mail_server`, Odoo usa ese y la copia manda correos de verdad. Un
> `UPDATE ir_mail_server SET active=false` a secas quita la barrera — medido en
> `tools/bench-neutralize`, y hasta el 2026-08-15 lo hacían tanto el SQL de extras como el propio
> gate, que exigía cero servidores activos. Igual con `base.autovacuum_job`: el estándar lo deja
> vivo a propósito.

### 4. Verificación final (gate de neutralización — duro)

No se entrega la BD al flujo hasta que TODOS estos checks pasen:

```bash
psql -U odoo ${CUSTOMER}_upgraded_base -tAc \
  "select value from ir_config_parameter where key='database.is_neutralized';"   # True
psql -U odoo ${CUSTOMER}_upgraded_base -tAc \
  "select count(*) from ir_mail_server where active=true and smtp_host <> 'invalid';"  # 0
psql -U odoo ${CUSTOMER}_upgraded_base -tAc \
  "select count(*) from ir_cron c
     where c.active and c.id not in (select res_id from ir_model_data
       where model='ir.cron' and name='autovacuum_job' and module='base');"          # 0
psql -U odoo ${CUSTOMER}_upgraded_base -tAc \
  "select count(*) from edi_backend where is_active=true;" 2>/dev/null || echo 0  # 0 (EDI Transus)
psql -U odoo ${CUSTOMER}_upgraded_base -tAc \
  "select count(*) from edi_backend where active=true;" 2>/dev/null || echo 0     # 0
```

Si cualquier check falla → corregir antes de continuar. Solo con todo en verde se
confirma que la BD `${CUSTOMER}_upgraded_base` está lista y se comunica a `odoo-database-upgrader`.
El conector EDI usa `is_active` (no `active`): verificar AMBAS columnas o una copia
de producción podría exportar EDI real (ver `references/neutralization.md`).

## Output

- BD local: `${CUSTOMER}_upgraded_base` (PostgreSQL, puerto 5432)
- BD desarrollo: `${CUSTOMER}_upgraded_dev` (copia de la anterior)
- Este dump se convierte en el `original.dump` inmutable del kit de migración (`odoo-database-upgrader` FASE 1.5)

## References

| Fichero | Contenido |
|---|---|
| `references/path-on-premise.md` | Pasos completos on-premise con comandos exactos |
| `references/neutralization.md` | **GATE de neutralización** — procedimiento + verificación dura + extras |
| `references/sql/neutralize_extras.sql` | SQL idempotente para conectores/credenciales no cubiertos por el estándar |
| `references/post-restore.md` | Post-restauración: neutralización, credenciales admin, copia dev |
| `common:references/shared/` | Convenciones compartidas cross-suite |

## GOTCHAS

1. **El dump llega SIN neutralizar (es de producción).** La neutralización NO es opcional ni
   "automática por versión". Ejecútala (`odoo-bin neutralize` + `neutralize_extras.sql`) y
   **verifícala con el gate duro ANTES de arrancar cualquier `odoo-bin`** — si no, mandas emails
   reales y disparas crons de producción.
2. **Odoo debe estar detenido antes de enviar la BD al Upgrade Service.** Si está corriendo durante
   el dump/upgrade puede enviar correos y dejar la BD en estado inconsistente.
3. **El Upgrade Service lee la suscripción enterprise de la propia BD** — no hace falta credencial
   aparte, pero si la suscripción no está en la BD el envío falla sin un error obvio.
4. **El upgrade puede tardar de minutos a horas** según el tamaño de la BD; el script bloquea hasta
   que Odoo devuelve el dump resultado. No lo mates pensando que está colgado.
5. **`neutralize_extras.sql` cubre conectores/credenciales que el `neutralize` estándar NO toca.**
   Si el cliente tiene integraciones (bancos, EDI, APIs), el `neutralize` de Odoo las deja activas;
   hay que pasar los extras o se conectan a sistemas reales.

## RULES

### NEVER
- Nunca arrancar `odoo-bin` sobre la BD restaurada antes de pasar el gate de neutralización.
- Nunca enviar la BD al Upgrade Service con Odoo corriendo.
- Nunca entregar `${CUSTOMER}_upgraded_base` al flujo si algún check del gate está en rojo.
- Nunca tratar la neutralización como "ya viene hecha" — siempre se ejecuta y se verifica.

### ALWAYS
- Confirmar `VERSION_DESTINO` y `CUSTOMER` antes de empezar.
- Verificar los 3 checks (`is_neutralized=True`, `ir_mail_server activos distintos del falso (host invalid) = 0`, `ir_cron activos salvo autovacuum = 0`)
  antes de comunicar que la BD está lista.
- Tratar el dump resultado como el `original.dump` inmutable del kit de migración.

## Delegates to

None — self-contained (solo usa `pg_dump`, el Upgrade Service y `psql`).

## Consumed by

| Skill | Suite | Context |
|-------|-------|---------|
| `dynapps-migrator` | developer | Orquestado en BLOQUE 2 (track de base de datos) |
| `odoo-database-upgrader` | developer | Consume `${CUSTOMER}_upgraded_base` como `original.dump` (FASE 1.5) |

See `common:references/shared/` for cross-suite shared conventions.
