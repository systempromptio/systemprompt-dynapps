# Lectura del log del servidor de Odoo

Como extraer informacion util del log para clasificar la familia (ver `error-families.md`) y localizar la causa. Soporta v17-v19.

## Donde esta el log

- **Por consola:** Odoo arrancado sin `--logfile` escribe a stdout/stderr.
- **`--logfile=/ruta/odoo.log`:** fichero en disco. En despliegues Indaws/Docker, mirar la config del servicio o los logs del contenedor.
- **Docker:** `docker logs <contenedor>` (anadir `2>&1 | tail -n 200`). Si el despliegue es de `docker-deployment`, el contenedor y compose estan parametrizados por proyecto.

Para el traceback solo importan las ultimas lineas relevantes: leer la **cola** del fichero (ultimas ~200 lineas) suele bastar. Para un fallo de `-u`, buscar hacia atras desde el final hasta el primer `Traceback (most recent call last)`.

## Anatomia de una linea de log

```
2026-06-19 10:12:33,456 12345 ERROR master odoo.modules.loading: Failed to load registry
```
`fecha hora  PID  NIVEL  base_de_datos  logger: mensaje`

- **NIVEL:** `CRITICAL` > `ERROR` > `WARNING` > `INFO` > `DEBUG`. El traceback va precedido de la linea `ERROR`/`CRITICAL` que lo dispara.
- **logger:** dice la fase. `odoo.modules.loading` / `odoo.modules.registry` = carga (`-u`/arranque). `odoo.sql_db` = SQL. `odoo.http` = peticion web/RPC. `werkzeug` = HTTP. `odoo.addons.<modulo>` = codigo de un modulo concreto.

## Subir el detalle del log

- `--log-level=debug` o `--log-level=debug_sql` (muestra cada query: util para B1/B2 y N+1, pero muy verboso).
- `--log-handler=odoo.addons.<modulo>:DEBUG` para subir solo un modulo.
- `--log-handler=odoo.sql_db:DEBUG` para ver el SQL exacto que provoca un IntegrityError/UndefinedColumn.

## Secuencia de carga del registry (clave para familias A y E)

Al instalar/actualizar, Odoo carga en este orden y el traceback de `-u` cae en alguno de estos pasos:

1. **Resolucion de `depends`** (orden topologico del manifest). Fallo aqui -> A4 (dependencia faltante).
2. **`_build_model` / `_setup_*`** (modelos, herencia, campos). Fallo -> A5 (KeyError/AttributeError de carga, `_inherit` a modelo no cargado).
3. **Creacion/actualizacion de columnas en BD.** Fallo -> B2 (UndefinedColumn si falta `-u`).
4. **Carga de `data` (XML/CSV) en el orden del manifest.** Fallo -> A1 (ParseError), A2 (validacion de vista), A3 (Field does not exist en vista/dato).
5. **Scripts de migracion** `migrations/<version>/pre|post|end-*.py`. Fallo -> E2. `pre` corre antes del paso 2; `post`/`end` despues del paso 4.

Implicacion: si el log dice que fallo cargando un `.xml`, mirar el **orden de `data`** y los `depends`, no solo el frame Python.

## Donde mirar segun el sintoma

| Sintoma en el log | Mirar primero |
|---|---|
| `ParseError: while parsing X.xml:NN` | esa linea del XML; despues `ref=`/external ids |
| `Field x does not exist` durante `-u` | manifest `depends` + orden `data` + el modelo destino |
| `psycopg2 ... constraint "nombre"` | mapear nombre -> `_sql_constraints`/FK; luego dato en vivo |
| `column x.y does not exist` | ¿campo nuevo sin `-u`? ¿`store=True` reciente? |
| `AccessError` en `odoo.http` | `ir.model.access.csv` + `ir.rule` + grupos del usuario |
| `Failed to load registry` | subir un nivel: el traceback real esta justo encima |
| log del servidor **limpio** pero UI rota | NO esta en el servidor -> consola del navegador (familia D) |

## Importante

- El log puede tener traceback **encadenado** (`During handling of the above exception, another exception occurred`). La causa raiz suele ser el **primer** traceback (el de mas arriba/mas interno), no el ultimo.
- `WARNING` repetidos antes del `ERROR` a menudo contienen la pista (ej. "field deprecated", "view inherits from missing").
- En `-u`, Odoo hace rollback de TODO el modulo al fallar: no asumas que lo cargado antes del error quedo aplicado.
