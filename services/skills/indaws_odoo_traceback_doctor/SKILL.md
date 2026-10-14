---
name: indaws-odoo-traceback-doctor
version: 0.1.8
effort: high
model: opus
description: >-
  Diagnostica un error Odoo en runtime desde el traceback o el log: -u que peta, IntegrityError, AccessError, assets/OWL. Usala cuando algo casque. Clasifica la familia, localiza la causa raíz y propone el fix mínimo. NOT auditoría estática (→ odoo-code-auditor).
allowed-tools: [Read, Grep, Glob, Bash, Skill, Agent]
---
<!-- verified: v18.0 @ 2026-06 -->

# Odoo Traceback Doctor

Diagnostica un fallo de Odoo en runtime partiendo de su traceback o log y devuelve causa raiz + fix minimo + como verificar.

Refs compartidas: `references/shared/odoo-connection.md` y `references/shared/batch-limits.md` (solo si hay que inspeccionar la instancia en vivo).

## Workflow

1. **Capturar el traceback/log.** Pide el traceback completo o la ruta al log del servidor. Si dan ruta, leelo (cola del fichero suele bastar: ultimas ~200 lineas). NO diagnostiques con una linea suelta: necesitas el tipo de excepcion, el mensaje y el frame mas profundo del codigo del proyecto. Para leer el log y entender niveles/secuencia de carga, ver `references/log-reading.md`.
2. **Clasificar la familia.** Identifica la familia del error por su firma (tipo de excepcion + mensaje + fase: instalacion/`-u`, runtime ORM, SQL, ACL, cliente/OWL, migracion). Catalogo en `references/error-families.md`.
3. **Localizar el codigo ofensor.** Con Grep/Glob/Read en el modulo del proyecto, encuentra el archivo y la linea reales (el ultimo frame que pertenece al codigo custom, no a `odoo/` ni a librerias). El traceback de Odoo a menudo apunta al sintoma, no a la causa: para `-u`/XML el origen suele estar en un `.xml`/manifest, no en el `.py` del frame.
4. **Explicar la causa raiz.** Di POR QUE Odoo falla aqui. [delega] Apoyate en `odoo-deep-knowledge` para comportamientos ocultos ("por que Odoo hace X") y en `odoo-version-api-reference` si la causa es un cambio de API entre v17/v18/v19. NO documentes API inline: referencia esas skills.
5. **Proponer el fix minimo.** El cambio mas pequeno que corrige la causa raiz, no un refactor. Patrones por familia en `references/fix-patterns.md`. Si el error revela deuda estatica mas amplia, [delega] sugiere `odoo-code-auditor` como paso aparte (no lo mezcles con el fix).
6. **Indicar como reproducir/verificar.** Comando exacto: re-`-u` del modulo, test dirigido, o `odoo shell`. Para inspeccionar datos/estado en vivo (constraints existentes, registros huerfanos, modulos instalados) [delega] en `odoo-pilot`. Da el criterio de exito ("arranca sin traceback", "la vista valida", "el constraint ya no salta").

## GOTCHAS

1. **El frame del traceback miente en errores de `-u`.** Un `ParseError` o `Field X does not exist` apunta al `.py` del ORM cargando la vista, pero la causa esta en el `.xml`/`.csv`/manifest. Busca el `name`/`model`/`arch` del registro que se estaba cargando, no el frame Python.
2. **"Field X does not exist" tiene tres causas distintas.** (a) typo/campo borrado, (b) el modulo que define el campo no esta en `depends` del manifest, (c) orden de carga: la vista se carga antes que el modelo. No asumas typo sin descartar dependencia y orden.
3. **`psycopg2.errors.UndefinedColumn` en arranque ≠ bug de codigo.** Suele ser una columna nueva aun no creada porque falta el `-u` del modulo, o un `store=True` anadido sin actualizar. Mirar si el campo es nuevo respecto a la BD antes de tocar Python.
4. **IntegrityError no dice que registro.** psycopg2 da el nombre del constraint, no el id. Hay que mapear el constraint -> modelo/campo y luego buscar el registro infractor en vivo (delega en odoo-pilot). El fix puede ser dato sucio, no codigo.
5. **AccessError vs ValidationError vs UserError piden fixes distintos** (aunque AccessError/ValidationError/MissingError heredan de UserError). AccessError = ACL/record rule (mirar `ir.model.access.csv` y `ir.rule`), no toques la logica. ValidationError/UserError = constraint o `raise` intencional en el codigo. Confundirlas lleva a "arreglar" seguridad relajando reglas. **En v19 la clase lleva su codigo HTTP** (`http_status`: 422/403/403/404, mas `LockError` 409) y hay dos tipos nuevos, asi que el codigo que ve el navegador ya te da la familia — no existe en v17/v18. Tabla en `references/error-families.md`.
6. **Errores de assets/OWL no salen en el log del servidor.** Estan en la consola del navegador. Si el sintoma es "la pantalla en blanco" o "el widget no carga" y el log del servidor esta limpio, pide la traza de la consola del cliente, no sigas en Python.
7. **Tras subir version, el `-u` falla por API retirada, no por tu cambio.** `name_get` retirado en v18 (deprecado con warning en v17), vistas con atributos eliminados, `@api.one`, etc. Antes de depurar como bug nuevo, delega en `odoo-version-api-reference` para ver que se retiro entre la version origen y destino.
8. **El loading order del registry importa.** `KeyError`/`AttributeError` durante carga suele ser que un modulo asume otro ya cargado. Revisar `depends` y `data` order en el manifest antes que el codigo.

## NEVER

- NUNCA diagnostiques con una sola linea del error; exige el tipo de excepcion y el frame del codigo del proyecto.
- NUNCA tomes el frame mas profundo del traceback como la causa si pertenece a `odoo/` o a una libreria; busca el ultimo frame del codigo custom.
- NUNCA propongas un refactor amplio como "fix": el fix es el cambio minimo que corrige la causa raiz.
- NUNCA documentes API de Odoo inline ni reimplementes la conexion: referencia `odoo-version-api-reference` y usa `odoo-pilot`/`common:references/shared/`.
- NUNCA relajes ACL/record rules para silenciar un AccessError sin confirmar que el acceso debe existir.
- NUNCA generes texto en espanol sin tildes (es_ES correcto). Codigo y nombres tecnicos en ingles.

## ALWAYS

- SIEMPRE clasifica la familia antes de buscar codigo (la familia dicta donde mirar).
- SIEMPRE distingue la fase del error: instalacion/`-u` vs runtime vs cliente vs migracion.
- SIEMPRE da el comando exacto para reproducir y el criterio de exito.
- SIEMPRE nota diferencias por version (v17/v18/v19) cuando la causa dependa de la version.
- SIEMPRE separa "fix del error" de "deuda detectada" (esta ultima -> delega en odoo-code-auditor).

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-version-api-reference | common | El error viene de un cambio de API entre v17/v18/v19 |
| odoo-deep-knowledge | (local) | Explicar comportamiento oculto: "por que Odoo hace X" |
| odoo-pilot | common | Inspeccionar datos/estado en vivo (constraints, registros, modulos) |
| odoo-code-auditor | developer | El error revela deuda estatica que merece auditoria aparte |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| - | - | - |

## REFERENCES

- `references/error-families.md` — catalogo de tracebacks tipicos de Odoo: firma/sintoma por familia y su causa.
- `references/log-reading.md` — como leer el log del servidor: niveles, `--log-level`, secuencia de carga del registry y donde mirar.
- `references/fix-patterns.md` — patrones de correccion por familia + como verificar cada uno.
