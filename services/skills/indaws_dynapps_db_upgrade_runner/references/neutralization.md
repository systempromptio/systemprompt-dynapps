# Neutralización de la BD restaurada — GATE OBLIGATORIO

> **Regla absoluta**: el dump migrado proviene de **producción**. NUNCA se arranca
> `odoo-bin` contra la BD restaurada hasta haber **verificado** que está neutralizada.
> No se asume neutralización automática. Se ejecuta y se comprueba siempre.

## Por qué no basta con "v17+ neutraliza solo"

La auto-neutralización de Odoo solo ocurre cuando **Odoo** duplica/clona una BD por
sus propios mecanismos. El dump que devuelve el **Upgrade Service** se restaura con
`psql`/`pg_restore` en crudo → **llega SIN neutralizar**, con todas las credenciales,
servidores de correo, crons y conectores de producción **activos**.

Arrancar Odoo contra esa BD sin neutralizar puede provocar, contra sistemas reales:
emails a clientes/proveedores, cobros, sincronizaciones bancarias, llamadas a EDI/transportistas.

## Paso 1 — Neutralizar (estándar Odoo)

Tras restaurar y **antes de cualquier arranque**, ejecutar la neutralización nativa.
Preferir el comando de Odoo (cubre todas las versiones y módulos instalados):

```bash
# Con odoo-bin del buildout (no arranca el servidor, solo neutraliza y sale)
./parts/odoo/odoo/odoo-bin neutralize -c odoo.cfg -d ${CUSTOMER}_upgraded_base --stop-after-init
```

Si el proyecto del cliente trae su propio SQL de post-clonado, ejecutarlo **además**:

```bash
psql ${CUSTOMER}_upgraded_base -f <ruta>/local/_tools/post_clonedb_local.sql
```

La neutralización estándar (v18: 65 ficheros `data/neutralize.sql`, uno por módulo) desactiva:
- `ir_mail_server` (correo saliente) y `fetchmail_server` (correo entrante)
- `ir_cron` — **todos menos `base.autovacuum_job`**, que deja vivo a propósito
- payment providers → modo `test`/`disabled`
- webhooks de `ir.actions.server` → URL inválida
- tokens IAP y `database.is_neutralized = true`

### Lo que hace y casi nadie sabe: DEJA un servidor de correo ACTIVO

`odoo/addons/base/data/neutralize.sql` desactiva los servidores reales y acto seguido **inserta uno
falso y activo**: `neutralization - disable emails`, host `invalid`, puerto 1025. Su propio
comentario lo explica: *"insert dummy mail server to prevent using fallback servers specified using
command line"*.

No es basura: es una **barrera**. Sin ningún `ir.mail_server`, Odoo cae al `smtp_server` del
`odoo.cfg` o de la línea de comandos
(`odoo/addons/base/models/ir_mail_server.py:445-451`, `IrMailServer.connect`; docstring de `send_email`, `:735-736`). Si ese
SMTP entrega de verdad, una copia de producción manda correos reales.

**Medido el 2026-08-15** (`tools/bench-neutralize`), estado correcto tras neutralizar:

| | Después del `neutralize` estándar |
|---|---|
| `ir_mail_server` activos | **1** — el falso, host `invalid` |
| `ir_cron` activos | **1** — `Base: Auto-vacuum internal data` |
| `database.is_neutralized` | `true` |
| claves de API en `ir_config_parameter` | **intactas** (de ahí el Paso 2) |

## Paso 2 — Neutralizar los EXTRAS que el estándar NO cubre

La neutralización nativa **no toca** credenciales que viven en módulos custom/OCA de
conectores ni en `server_environment`. Ejecutar siempre `neutralize_extras.sql`:

```bash
psql ${CUSTOMER}_upgraded_base -f <SKILL_DIR>/references/sql/neutralize_extras.sql
```

Cubre, de forma idempotente y defensiva (no falla si la tabla no existe):
- Conectores EDI (`*connector_edi*`, `alpiek_connector`) → endpoints/credenciales a vacío
- Sincronización bancaria (`account_online_synchronization`) → desactivada
- Transportistas (`delivery_bpost`, `delivery_ups`, etc.) → credenciales a test/vacío
- `server_environment` parámetros sensibles → forzar entorno `dev`/local
- `ir_config_parameter` con claves `*api*key*`, `*secret*`, `*token*`, `*password*` de servicios externos → blanqueadas
- Webhooks salientes / base URLs de producción → `http://localhost:8069`

> Revisar el output del script: lista cada tabla/clave tocada. Si aparece un conector
> ZNU específico no contemplado, **STOP** y ampliar el SQL antes de continuar.

## Paso 3 — VERIFICACIÓN (gate duro)

No se continúa hasta que TODAS estas comprobaciones pasen:

```bash
# 1. Marca de neutralización presente
psql ${CUSTOMER}_upgraded_base -tAc \
  "select value from ir_config_parameter where key='database.is_neutralized';"
# Debe devolver: True

# 2. Cero servidores de correo REALES activos...
psql ${CUSTOMER}_upgraded_base -tAc \
  "select count(*) from ir_mail_server where active=true and smtp_host <> 'invalid';"
# Debe devolver: 0

# 2b. ...y el servidor falso TIENE que seguir ahí (es la barrera anti-fallback)
psql ${CUSTOMER}_upgraded_base -tAc \
  "select count(*) from ir_mail_server where active=true and smtp_host = 'invalid';"
# Debe devolver: 1   ← si devuelve 0, alguien lo borró: repón el dummy antes de arrancar

# 3. Cero crons activos salvo el autovacuum, que el estándar respeta a propósito
psql ${CUSTOMER}_upgraded_base -tAc \
  "select count(*) from ir_cron c
     where c.active and c.id not in (select res_id from ir_model_data
       where model='ir.cron' and name='autovacuum_job' and module='base');"
# Debe devolver: 0

# 4. Sin sincronización bancaria viva
psql ${CUSTOMER}_upgraded_base -tAc \
  "select count(*) from account_online_link;" 2>/dev/null || echo "0 (tabla ausente, OK)"
# Debe devolver: 0 (o tabla ausente)

# 5. Conectores EDI desactivados (flag varía: is_active y/o active)
#    ⚠ Un edi_backend vivo auto-exporta albaranes/facturas de VERDAD (ZNU).
psql ${CUSTOMER}_upgraded_base -tAc \
  "select count(*) from edi_backend where is_active=true;" 2>/dev/null || echo "0 (col/tabla ausente, OK)"
psql ${CUSTOMER}_upgraded_base -tAc \
  "select count(*) from edi_backend where active=true;" 2>/dev/null || echo "0 (col/tabla ausente, OK)"
# Ambos deben devolver: 0 (o col/tabla ausente)
```

Si **cualquiera** falla → re-ejecutar Paso 1/2 o corregir a mano. **Nunca** arrancar Odoo con un check en rojo.

> **Ojo con el flag del EDI**: `edi_backend` (conector Transus) neutraliza por
> `is_active`, no por `active`. Un `SET active=false` deja el conector VIVO y una
> copia de producción puede exportar EDI real. Por eso el gate comprueba las dos
> columnas y `neutralize_extras.sql` pone a false la que exista.

## Paso 4 — Restablecer credenciales admin (tras verificar neutralización)

```sql
-- Confirmar primero quién es admin
select id, login from res_users where active = true order by id limit 5;
-- Resetear (ajustar id si admin != 2)
update res_users set password='admin', login='admin' where id=2;
```

## Resumen del orden inviolable

```
restaurar dump → Paso 1 (neutralize) → Paso 2 (extras) → Paso 3 (VERIFICAR) → Paso 4 (admin)
                                                              │
                                          ❌ check en rojo ───┘ → corregir, NO arrancar Odoo
                                          ✅ todo verde ──────────→ ya se puede arrancar odoo-bin
```
