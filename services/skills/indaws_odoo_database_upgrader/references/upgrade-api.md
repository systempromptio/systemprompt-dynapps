# Odoo Upgrade Service - CLI Reference

> **IMPORTANTE**: La antigua API REST HTTP (`/database/v1/create`, `/upload`, `/status`, `/download`)
> fue eliminada por Odoo. El servicio ahora funciona exclusivamente via un script CLI Python
> que gestiona la transferencia de datos por SSH/rsync.

## Comando base

```bash
python3 <(curl -s https://upgrade.odoo.com/upgrade) <subcomando> [opciones]
```

El script se descarga y ejecuta directamente desde `https://upgrade.odoo.com/upgrade`.

## Subcomandos disponibles

| Subcomando | Funcion |
|------------|---------|
| `test` | Upgrade de prueba (ilimitados por contrato) |
| `production` | Upgrade de produccion (1 por contrato y BD) |
| `restore` | Descargar y restaurar un upgrade anterior por token |
| `status` | Consultar estado de una peticion por token |
| `log` | Ver log de la peticion por token |
| `wipe` | Eliminar todos los dumps del contrato en la plataforma |

Opcion global: `--debug` para activar trazas de depuracion.

## Subcomando `test` / `production`

Son identicos en opciones. `test` permite envios ilimitados; `production` es 1 por contrato.

```bash
python3 <(curl -s https://upgrade.odoo.com/upgrade) test \
  -i dumps/to_upload.dump \
  -c M12345-67890 \
  -t 18.0 \
  -x
```

### Opciones

| Flag | Largo | Requerido | Descripcion |
|------|-------|-----------|-------------|
| `-d` | `--dbname` | Si* | Nombre de BD local a dumpear y subir |
| `-i` | `--dump` | Si* | Ruta a dump existente (.sql, .dump, .sql.gz, .zip, dir con toc.dat) |
| `-c` | `--contract` | Si con `-i` | Contrato Enterprise (ej: `M12345-67890`). Con `-d` se lee de la BD |
| `-t` | `--target` | Si | Version destino (ej: `18.0`) |
| `-r` | `--restore-name` | No | Nombre BD local para restaurar resultado. No crear manualmente |
| `-x` | `--no-restore` | No | Descargar ZIP sin restaurar localmente |
| `-e` | `--env` | No | Variable de entorno `VAR=VAL` para el proceso |
|  | `--env-file` | No | Fichero con variables de entorno (una por linea, `VAR=VAL`) |
| `-s` | `--ssh-key` | No | Clave SSH para transferencia (default: `/tmp/<UID>_upgrade_ssh_key`) |
| `-j` | `--core-count` | No | Nucleos para dump/restore paralelo (default: 4) |

*`-d` y `-i` son mutuamente excluyentes. Uno de los dos es obligatorio.

### Modos de entrada

| Flag | Caso de uso | Notas |
|------|-------------|-------|
| `-d DBNAME` | BD PostgreSQL local accesible | El script hace el dump automaticamente. Contrato se lee de la BD |
| `-i DUMP` | Dump ya generado | Requiere `-c` obligatorio. Ideal para nuestro workflow con Docker |

**En nuestro workflow siempre usamos `-i` con dump pre-generado desde el contenedor Docker.**

### Que hace el script internamente

1. Genera clave SSH temporal (si no existe)
2. Sube el dump via rsync/SSH al servidor de Odoo
3. Inicia la migracion en el servidor
4. Muestra progreso en tiempo real (saltos de version, tests, warnings)
5. Al terminar, descarga el resultado como `upgraded.zip` via rsync
6. Si no se uso `-x`: restaura la BD localmente

### Salida del script

El script muestra en stdout:
- Token de la peticion y servidor asignado (ej: `eu3.upgrade.odoo.com`)
- Progreso por cada salto de version (ej: 15.0→16.0→17.0→18.0)
- Resultado de tests (failures, errors, total)
- Warnings (campos duplicados, modelos sin acceso, etc.)
- Tiempo total de procesamiento

El token se guarda automaticamente en `upgrade.log` en el directorio de ejecucion.

**Exit codes:**
- `0` = exito (upgrade completado, dump descargado)
- `!= 0` = fallo (error en migracion, test fallido, timeout, etc.)

### Resultado descargado

El fichero `upgraded.zip` se descarga al directorio de ejecucion y contiene:
- `dump.sql` — BD migrada en SQL plano (restaurar con `psql`)
- `filestore/` — Filestore **parcial**: solo ficheros nuevos de la version destino
  (iconos, logos proveedores de pago, imagenes estandar Odoo).
  NO contiene los adjuntos del cliente.

## Subcomando `restore`

Descarga y restaura un upgrade anterior usando su token.

```bash
python3 <(curl -s https://upgrade.odoo.com/upgrade) restore \
  -t <TOKEN> \
  -d <DBNAME_ORIGINAL> \
  -r <NOMBRE_BD_RESTAURADA>
```

| Flag | Largo | Requerido | Descripcion |
|------|-------|-----------|-------------|
| `-t` | `--token` | Si | Token de test/production (de stdout o `upgrade.log`) |
| `-d` | `--dbname` | Si | BD local original (para obtener filestore) |
| `-r` | `--restored-name` | Si | Nombre BD destino para restaurar |
| `-x` | `--no-restore` | No | Solo descargar ZIP sin restaurar |
| | `--production` | No | Indica que es BD de produccion (no test) |
| `-s` | `--ssh-key` | No | Clave SSH |
| `-j` | `--core-count` | No | Nucleos para restore |

## Subcomando `status`

Consulta el estado de una peticion.

```bash
python3 <(curl -s https://upgrade.odoo.com/upgrade) status -t <TOKEN>
```

| Flag | Largo | Requerido | Descripcion |
|------|-------|-----------|-------------|
| `-t` | `--token` | Si | Token de la peticion |

## Subcomando `log`

Muestra el log completo de la migracion.

```bash
python3 <(curl -s https://upgrade.odoo.com/upgrade) log -t <TOKEN>
```

| Flag | Largo | Requerido | Descripcion |
|------|-------|-----------|-------------|
| `-t` | `--token` | Si | Token de la peticion |
| `-f` | `--from-byte` | No | Byte de inicio del log (0 = desde el principio) |

## Subcomando `wipe`

Elimina TODOS los dumps del contrato en la plataforma. **Accion destructiva.**

```bash
python3 <(curl -s https://upgrade.odoo.com/upgrade) wipe -t <TOKEN> -c <CONTRACT> -y
```

| Flag | Largo | Requerido | Descripcion |
|------|-------|-----------|-------------|
| `-t` | `--token` | Si | Token de la ultima peticion |
| `-c` | `--contract` | Si | Contrato asociado |
| `-y` | `--yes` | No | Pre-aprobar sin confirmacion interactiva |

## Filestore: input vs output

| | Lo que se envia | Lo que devuelve el servicio |
|---|---|---|
| **Dump BD** | `.dump` (pg_dump -Fc) o `.sql` | `dump.sql` dentro del ZIP |
| **Filestore** | No se envia | Parcial (solo ficheros nuevos de Odoo) |
| **Adjuntos cliente** | No aplica | NO incluidos — copiar manualmente |

**Gestion del filestore en el despliegue:**

```
filestore_final/
├── [filestore original del cliente]   ← copiar desde instalacion v_origen
└── [filestore parcial del ZIP]        ← merge con lo devuelto por el servicio
```

## Formato de dump recomendado

**Usar siempre `pg_dump -Fc` (formato custom binario).**

```bash
docker exec odoo-upgrade-analyzer-pg{PG} pg_dump -U odoo -Fc -d <db_name> > dumps/to_upload.dump
```

| Criterio | `.sql` (texto plano) | `.dump` (custom -Fc) |
|---|---|---|
| Tamano tipico | 100% referencia | ~20-30% del SQL |
| Velocidad de subida | Lenta | 3-5x mas rapido |
| Riesgo de superar limites | Alto en BDs grandes | Bajo |
| Soportado por CLI | Si | Si |

**Formatos aceptados por el CLI:** `.sql`, `.dump`, `.sql.gz`, `.zip`, directorio psql con `toc.dat`.

## Limites y restricciones

| Limite | Valor |
|--------|-------|
| Requests test simultaneos | Sin limite |
| Requests produccion | 1 por contrato y BD |
| Tiempo maximo procesamiento | ~24 horas |
| Retencion del resultado | 30 dias |
| Versiones soportadas destino | 16.0 - 19.0 |

## Errores frecuentes

| Error | Causa | Solucion |
|-------|-------|----------|
| Contrato no valido | Contrato expirado o incorrecto | Verificar con Odoo SA |
| Test fallido (exit code != 0) | Error en migracion o tests | Revisar log, crear fix SQL, reintentar |
| Timeout SSH | Conexion lenta o firewall | Verificar red, reintentar |
| Dump demasiado grande | BD muy grande | Limpiar BD, VACUUM FULL, regenerar dump |

## Gotchas operativos (origen: Pedak v16→v19)

Comportamientos del Upgrade Service que sorprenden la primera vez y
que **conviene saber antes** de empezar:

| Gotcha | Impacto | Workaround |
|---|---|---|
| **Custom modules SKIPPED**. El servicio corre Odoo solo con CE+EE de su pipeline; `parts/custom/` y `parts/oca_pull/` nunca se ven. | Los `pre-migrate.py`/`end-migrate.py` de tus custom NO se ejecutan durante el upgrade. La mayoría del "cleanup v_(N+M)" es post-restore, no migración del módulo. | Replicar el efecto en SQL post-restore. El kit cubre esto vía `post_restore_validator` + `legacy_to_studio_copy` + `successor_marker`. |
| **Filestore en DOS fuentes**. (a) Filestore original del cliente NO está en el dump — transferir aparte. (b) `upgraded.zip` trae filestore **parcial** (solo ficheros regenerados durante la migración). | Hay que **merge**: primero el original, después overlay del parcial. Verificar abriendo un registro con attachment conocido. | `cp -r original/. dest/ && cp -r partial/. dest/`. El helper `FilestoreExtractor` del kit hace el segundo paso. |
| **Cada iteración resetea**. Re-subir tras fallo arranca desde el dump v_N **original**, no del estado del intento anterior. | No puedes "parchar el dump medio-migrado". Hay que aplicar el fix en v_N antes de re-subir. | SQL numerados `sql/01_*.sql`, `sql/02_*.sql`, etc. aplicados en orden antes de cada upload. |
| **Tax-tag balance check puede bloquear submission**. El servicio valida que los tax-tag balances de NL/ES reconcilien tras el re-mapping; un desequilibrio real falla el run. | Si el desequilibrio es aceptable (decisión contabilidad), usar bypass. | Env var `ODOO_UPG_IGNORE_TAX_TAGS=1` al subir. Documentar el desequilibrio en el plan. |
| **Algunas schema migrations quedan a medias**. Ej. `ir_model_inherit.parent_id` mantiene su `NOT NULL` v16 aunque v19 mete NULL ahí en modelos abstractos. El dump no arranca v_(N+M) Odoo sin este fix. | El dump migrado NO significa "arranca limpio en v_(N+M)". Hay que fix de schema post-restore. | `ALTER TABLE ir_model_inherit ALTER COLUMN parent_id DROP NOT NULL; DELETE FROM ir_model_inherit WHERE parent_id IS NOT NULL AND parent_id NOT IN (SELECT id FROM ir_model);` (candidato a helper kit). |
| **Estados de módulo en formas inesperadas**. Algunos installed en v_N vuelven como `uninstalled` (porque el código v_(N+M) no se envió), otros siguen `installed` pero a versión vieja. | Validación post-restore debe verificar: ¿módulos en `to upgrade`/`to install`/`to remove`? ¿Algún `installed` a versión v_N? ¿Algún stub showing `installed`? | El kit valida esto en `post_restore_validator.validate_modules` (P7 cubre los `ir.model.fields` huérfanos del caso "installed at v_N version"). |

## Gotchas de infraestructura local

| Gotcha | Workaround |
|---|---|
| Odoo local NO genera PDFs sin `wkhtmltopdf` (build patcheado específico). Falla silenciosa: cae a HTML, oculta errores que solo aparecen en PDF | Test renders via `env['ir.actions.report']._render_qweb_html(...)` desde `odoo-bin shell` — mismo pipeline. |
| Browser cachea OWL assets agresivamente tras edits JS | Hard-refresh Ctrl+Shift+R tras cambios JS; si no, validas versión vieja y piensas que tu fix no funciona. |
| Tras `neutralize_extras.sql`, todos los users tienen password `admin` y TOTP limpio | Mencionar en README cliente; agentes no "investigar" errores de auth que vengan de neutralize. |

## Filestore: workflow completo

> Cubierto parcialmente por `helpers/exec/filestore_extractor.py` del
> kit, pero el merge de las DOS fuentes es manual:

1. **Original del cliente** (desde su prod, vía SCP/rsync):
   ```bash
   rsync -av cliente@prod:/var/lib/odoo/filestore/<db>/ \
         ~/INDAWS/<cliente>/filestore_original/
   ```
2. **Parcial del Upgrade Service** (extraído de `upgraded.zip`):
   ```bash
   indaws-migrate run --apply  # incluye FilestoreExtractor si está en plan
   ```
3. **Merge** (original primero, parcial overlay):
   ```bash
   cp -r ~/INDAWS/<cliente>/filestore_original/. \
         /var/lib/docker/volumes/<cliente>_filestore/_data/
   # Luego el FilestoreExtractor del kit ya puso el parcial encima.
   ```

Verificar: abrir un partner con logo conocido → debe mostrar la imagen.
