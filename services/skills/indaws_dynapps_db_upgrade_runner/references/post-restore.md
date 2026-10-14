# Post-Restauración (todos los paths)

Pasos comunes tras restaurar el dump migrado en PostgreSQL local, independientemente del path usado.

> ⚠️ El dump viene de **producción** y el Upgrade Service lo devuelve **SIN neutralizar**.
> La neutralización es un **GATE OBLIGATORIO** antes de arrancar Odoo. No se asume
> "auto-neutralización por versión". Procedimiento completo y verificación dura en
> **`neutralization.md`** — leerlo y ejecutarlo entero.

## 1. Neutralizar (GATE OBLIGATORIO)

**Antes de cualquier arranque de `odoo-bin`.** Resumen (detalle en `neutralization.md`):

```bash
# a) Neutralización nativa de Odoo
./parts/odoo/odoo/odoo-bin neutralize -c odoo.cfg -d ${CUSTOMER}_upgraded_base --stop-after-init

# b) SQL del cliente si existe
psql ${CUSTOMER}_upgraded_base -f <ruta>/local/_tools/post_clonedb_local.sql   # si aplica

# c) Extras que el estándar NO cubre (conectores EDI, banca, transportistas, secrets)
psql ${CUSTOMER}_upgraded_base -f <SKILL_DIR>/references/sql/neutralize_extras.sql
```

## 2. Verificar neutralización (gate duro — no continuar si falla)

```bash
psql ${CUSTOMER}_upgraded_base -tAc "select value from ir_config_parameter where key='database.is_neutralized';"  # True
psql ${CUSTOMER}_upgraded_base -tAc "select count(*) from ir_mail_server where active=true;"                      # 0
psql ${CUSTOMER}_upgraded_base -tAc "select count(*) from ir_cron where active=true;"                             # 0
```

Si **cualquier** check no da el valor esperado → re-ejecutar Paso 1 o corregir a mano.
**Nunca arrancar Odoo con un check en rojo.** Ver `neutralization.md` §Paso 3 para la batería completa.

## 3. Restablecer credenciales admin

```bash
psql ${CUSTOMER}_upgraded_base
```

Dentro de psql:

```sql
update res_users set password = 'admin' where id = 2;
update res_users set login = 'admin' where id = 2;
\q
```

> Verificar que el usuario con `id = 2` es efectivamente el admin. Si la BD usa otro id para admin,
> usar: `select id, login from res_users where active = true order by id limit 5;`

## 4. Crear copia para desarrollo

```bash
createdb -T ${CUSTOMER}_upgraded_base ${CUSTOMER}_upgraded_dev
```

## 5. Verificación final

```bash
psql -l | grep ${CUSTOMER}_upgraded
```

Confirmar que existen ambas BDs:
- `${CUSTOMER}_upgraded_base` — copia inmutable de referencia
- `${CUSTOMER}_upgraded_dev` — copia de trabajo para el equipo de desarrollo

## Output para odoo-database-upgrader

- `${CUSTOMER}_upgraded_base` se convierte en el `original.dump` inmutable del kit:
  ```bash
  pg_dump -Fc ${CUSTOMER}_upgraded_base > /path/to/migration_kit/original.dump
  ```
- Este dump se usa en `odoo-database-upgrader` FASE 1.5 (`indaws-migrate init`)
