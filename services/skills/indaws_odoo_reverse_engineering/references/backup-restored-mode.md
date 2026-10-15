# Modo "backup restaurado" (filesystem + sandbox)

El flujo base de la skill asume el caso más pobre: **solo lectura por API contra producción**. Ahí
el `.pyc` es una caja negra y ejecutar flujos "ya no es solo lectura", así que queda fuera.

Pero muchas veces el cliente puede facilitar un **backup restaurable**. Eso cambia el juego: tienes
(a) el **filesystem** con los `.pyc` y (b) una **BD desechable** que puedes machacar en un entorno
aislado. Se desbloquean tres canales que suben masa de "no determinable" a "confirmado/observado".

Este modo es **aditivo**: haz primero el flujo base (pasos 1-7b) y luego añade lo que aplique.

## Canal A — Análisis estático del bytecode (tienes los `.pyc`)

Aunque no siempre se pueda decompilar a fuente, el bytecode se lee con la stdlib en **cualquier**
versión de Python. Sin ejecutar nada:

```bash
python3 scripts/static_pyc.py <out_dir> <ruta_addons_del_backup> [<mod> ...]
```

Genera `<mod>.static.json` con: **imports externos** (revelan integraciones: `requests`=HTTP,
`zeep`=SOAP, `paramiko`=SFTP, `cryptography`/`xmlsig`=firma…), **strings** (SQL crudo, URLs,
claves de config, mensajes), **métodos ORM** llamados (`create`/`write`/`action_*`/`_compute_*`),
**funciones definidas** y una señal de **módulo licenciado/ofuscado** (`licenciado_probable`).

- **Decompilación (opcional):** si la versión lo permite —Python ≤3.8 (Odoo 14-15) con
  `decompyle3`/`uncompyle6`, o `pycdc` (parcial) en 3.10+— intenta recuperar el fuente. Lo que salga
  es **aproximación**, nunca confirmado, y va etiquetado.
- **Overrides sin footprint:** el custom más traicionero redefine métodos estándar
  (`action_confirm`, `_prepare_invoice`, `create`…) **sin añadir ni modelo ni campo** → footprint
  estructural cero. Los metadatos son ciegos a esto; solo lo cazan `static_pyc.py` (nombres de
  función que pisan métodos estándar) y el diff de comportamiento del Canal B. Búscalo explícitamente.
- **Módulos licenciados/ofuscados:** si `licenciado_probable` es cierto, decláralo como **límite**
  del análisis (no se ejecutan sin key ni decompilan) y no pierdas tiempo peleándolo.

## Canal B — Ejecución instrumentada en sandbox (la BD es desechable)

El backup restaurado se puede **ejecutar de verdad**, con las integraciones externas cortadas. Esto
es lo que convierte "observado en un rango" en "observado en todo el dominio".

**Reglas de oro del sandbox:**
- **Aislado y stubbeado:** integraciones externas (AEAT, SERES, bancos, carriers) mockeadas de forma
  determinista; nada sale a producción de terceros. Requiere OK del cliente (ya no es solo lectura).
- **Reproducible:** congela fecha/hora y secuencias, o el comportamiento no se puede repetir ni
  verificar.

Técnicas, de menor a mayor esfuerzo:
1. **`--log-level=debug --log-sql`** + subir `ir.logging`: captura el SQL exacto que emite el código
   compilado y qué corre en cada flujo.
2. **Instrumentación por monkeypatch al arranque** (addon server-wide con `--load`): envuelve
   `create/write/unlink` y los `compute` para loguear entradas→salidas reales. Aquí `ir.logging` **sí**
   se enciende aunque el código no loguee, porque lo inyectas tú.
3. **`sys.settrace`/coverage** durante un tour de UI: mapea botón→funciones que se ejecutan.
4. **Active learning (diseño de experimentos):** genera TÚ combinaciones de entrada que la BD nunca
   ejerció —fronteras y casos extremos donde una fórmula candidata podría romperse—, ejecuta el
   proceso y compara el **antes/después de los datos**. Es lo que valida las fronteras que el muestreo
   pasivo no cubre.
5. **`@api.onchange` (punto ciego):** lógica solo de UI, no persiste ni corre en batch → **invisible**
   a la observación de datos. Solo la ves con el Canal A o ejecutando la UI aquí. Decláralo como clase
   no cubrible por el canal de datos si no se ejecuta la UI.

## Canal C — Inferencia de fórmula (búsqueda acotada de hipótesis)

La mayoría de la lógica a medida es aritmética. Tras `analyze_behavior.py`, intenta **recuperar la
fórmula** de cada campo calculado numérico con una medida de fiabilidad honesta. Ojo con el nombre:
esto **no es regresión simbólica plena** (no usa PySR/gplearn); es una **búsqueda sobre familias de
hipótesis curadas** con criterio de ajuste exacto. Es deliberadamente conservador —prefiere no
proponer nada a proponer una fórmula que solo "ajusta bien":

```bash
python3 scripts/infer_formula.py <out_dir> [--samples 500]
```

Genera `<mod>.formulas.json`. Solo acepta candidatas con **ajuste exacto** (residuo ~0 en toda la
muestra), reporta **holdout** (ajuste en 70% / test en 30%), **dominio observado** de cada entrada, y
una **cota rule-of-three** (0 fallos en N ⇒ error ≤ ~3/N al 95%, condicional al dominio). Familias:
identidad, suma, producto (·k), porcentaje/descuento, ratio, combinación lineal.

**Mejora futura (no implementada):** regresión simbólica plena (PySR) para fórmulas fuera de estas
familias, y *model-tree* (árbol de decisión que descubre fronteras + inferencia por hoja) para lógica
por tramos (IRPF, rappels escalados). Hoy, si ves saltos, separa a mano por el campo condición.

- **Sigue siendo "comportamiento observado", nunca "confirmado"** — es fórmula inferida, no leída.
- **No-determinismo:** si las mismas entradas producen salidas distintas (depende de fecha, usuario,
  secuencia, estado externo), el script lo marca `no_determinista_probable` y **no** fuerza fórmula.
- **Tramos (if/else):** para IRPF, rappels escalados, etc., el patrón correcto es *model tree* —árbol
  de decisión que descubre las fronteras + regresión por hoja. `infer_formula.py` no lo hace aún; si
  ves saltos, sepáralos a mano por el campo condición y corre la inferencia por tramo.
- **Aislar el delta custom:** si tienes el Odoo limpio (Canal D), lo ideal es regresar sobre la
  **diferencia** cliente−vanilla, no sobre la salida bruta (que mezcla estándar + custom).

## Canal D — Diff contra un Odoo limpio

Sustituye la heurística de `discover_modules.py` por evidencia. Levanta un Odoo **vanilla** de la
misma versión (+ los OCA que use el cliente) y diferencia estructura:

```bash
# en la BD del cliente:
python3 scripts/dump_fields.py fields_cliente.json --out-dir <out_dir>
# en la BD vanilla (mismas variables ODOO_* apuntando al limpio):
python3 scripts/dump_fields.py fields_limpio.json --all-models
# diff:
python3 scripts/baseline_diff.py fields_cliente.json fields_limpio.json diff.json
```

`diff.json` da modelos nuevos, campos nuevos por modelo (extensiones) y campos con tipo/almacenamiento
cambiado — exactamente lo que es a medida, sin adivinar.

## Cobertura extra (vale también en modo API)

`scripts/extract_extras.py <out_dir> [<mod>...]` añade lo que la estructura básica no cubre y que la
API expone como texto legible o metadatos: **informes QWeb** (a menudo *son* el desarrollo),
**acciones automatizadas** (`base.automation`: disparador + código), **integraciones**
(`ir.config_parameter`, crons), **footprint de `ir.model.data`** (qué datos siembra el módulo) y
**seguridad** (`ir.model.access` + `ir.rule`). Es read-only y conviene correrlo siempre.

## Encaje en el flujo y triaje de coste

- Canal A, D y `extract_extras` son **baratos** → hazlos siempre que tengas filesystem/limpio.
- El Canal B (sandbox) es **caro** (restaurar + ejecutar). Reserva la instrumentación pesada para los
  módulos de **alto uso × alta incertidumbre** (usa la ponderación por registros de `consolidate.py`
  para decidir dónde gastarlo). El resto se queda en estático + observado.
- **Palimpsesto de versiones:** los datos son de varias generaciones del código (el módulo se
  "actualizó" durante años). Registros viejos se calcularon con código que ya no existe → la regresión
  ve contradicciones. Pondera/segmenta por `write_date` y, para campos `store`, recuerda que el valor
  guardado puede estar **obsoleto** (calculado con código antiguo): observar el dato ≠ la lógica actual.
