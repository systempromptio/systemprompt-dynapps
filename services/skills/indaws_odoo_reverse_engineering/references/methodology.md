# Metodología

## Qué SÍ se puede leer y qué NO

El código Python compilado (`.pyc`) no es legible. Pero muchísima información del comportamiento
del sistema **vive en la base de datos** y se lee con métodos inofensivos:

| Se lee con certeza (en BD / registro vivo) | NO es legible (compilado) |
|---|---|
| Modelos, campos, tipos, relaciones (`ir.model`, `ir.model.fields`) | La fórmula exacta de un método `compute` |
| Vistas, menús, informes, secuencias, plantillas | Los pasos internos de un botón `type=object` |
| **Código** de acciones de servidor y crons (texto plano en BD) | La lógica interna de una acción automatizada compilada |
| Permisos (`ir.model.access`), reglas (`ir.rule`), restricciones SQL | El cuerpo de los métodos del modelo Python |
| Dependencias reales de campos calculados (`fields_get` → `depends`) | |
| Nº de registros por modelo (`search_count`) = uso real | |

La clave: de un cálculo casi siempre conocemos **qué entra** (`depends`) y **qué sale** (el campo
y su tipo), aunque no la operación intermedia. Eso se reporta como entradas/salidas, no como fórmula.

**Si hay backup restaurado** la columna "NO legible" se encoge: el bytecode `.pyc` se lee con la
stdlib (imports, strings/SQL, métodos, overrides), la fórmula aritmética se puede **inferir** de
datos (búsqueda acotada de hipótesis, ver Canal C) y el comportamiento se puede **ejecutar** en sandbox. Ver
`backup-restored-mode.md`. Aun así, inferir ≠ leer: la fórmula recuperada sigue siendo
"comportamiento observado", nunca "confirmado".

## «Solo lectura» tiene dos significados, y solo uno es cierto aquí

**Medido el 2026-08-15 en v18 y v19** (`tools/bench-orm`, `test_readonly.py`).

`scripts/_odoo.py` **sí** hace lo que dice: solo deja pasar diez métodos
(`search`, `search_read`, `read`, `fields_get`, `search_count`, `read_group`, `name_search`,
`default_get`, `get_views`, `fields_view_get`) y cualquier otro lanza `PermissionError`. Verificado
también que **ningún script habla con Odoo por su cuenta**: todos pasan por el wrapper, y los cinco
que no lo importan (`build_facts`, `consolidate`, `build_report`, `baseline_diff`, `static_pyc`)
trabajan sobre ficheros locales.

Lo que **no** se sigue de ahí es que la operación no escriba nada. Esos diez métodos **ejecutan
código del cliente** — que es código a medida, compilado y desconocido: exactamente lo que estás ahí
para averiguar. Con un modelo de sonda que escribe en cada gancho, y usando **solo** métodos de la
lista blanca, se midió escritura por siete caminos distintos:

| Lo que pides | Qué ejecuta del cliente |
|---|---|
| `search_read` / `read` de un campo calculado | su método `compute` |
| `search` o `search_count` con un dominio sobre un campo con `search=` | su método `_search_…` |
| `name_search` | su `_compute_display_name` — **no** su `_name_search`, que ya no existe (v18 `odoo/models.py:1897`, `BaseModel.name_search`, v19 `odoo19/odoo/orm/models.py:1542`, `BaseModel.name_search`: se busca sobre `display_name`) |
| `default_get` | su override de `default_get` |
| `fields_get` | su override de `fields_get`, si lo tiene |
| `read_group` agrupando por un calculado almacenado | el `compute` que rellene lo pendiente |

Y un caso que no necesita código hostil, solo el ORM: **un campo almacenado marcado para recalcular
se ESCRIBE en la base en cuanto alguien lo lee**. En la sonda, la columna tenía `999`, se leyó el
campo y quedó `6`. Un `read()` produjo un `UPDATE`.

**Qué hacer con esto, sin dejar de trabajar:**

- La frase honesta ante el cliente no es «no vamos a escribir nada», es **«no vamos a modificar
  datos de negocio a propósito, y usamos solo lecturas; leer un sistema hace que el sistema calcule,
  y eso lo escriben sus propios desarrollos, no nosotros»**.
- El riesgo que de verdad importa **no revierte con un rollback**: un `compute` a medida que llame a
  un servicio externo (facturación electrónica, mensajería, un ERP de terceros) se dispara al leer.
  Antes de `analyze_behavior.py --samples 200` sobre producción, mira qué campos vas a muestrear y
  pregunta si alguno tiene integración detrás. Es la pregunta que hay que hacer, y cuesta un minuto.
- Prefiere campos **almacenados** para muestrear: se leen de la columna. Los `compute` sin
  `store=True` ejecutan siempre.
- Si el cliente exige cero escrituras verificables, eso solo se garantiza en **modo backup**
  restaurado, o con un usuario de base de datos de solo lectura — no con una lista blanca de
  métodos.

## Regla de certeza — cuatro niveles

El cliente exige rigor. Clasifica cada pieza de lógica en exactamente uno de estos niveles:

- **Confirmado** — certeza del 100 %. Estructura, entradas/salidas de un cálculo, código legible
  de acciones de servidor/crons, opciones reales de los campos de selección.
- **Comportamiento observado** — lo que un cálculo PRODUCE se ve claro y exacto en datos reales
  (`analyze_behavior.py`): la relación entrada→salida se cumple en toda la muestra, sin contraejemplos.
  Se cita el nº de registros. Es **evidencia empírica, NO la fórmula del código** — dilo siempre así.
- **Aproximación** — se intuye con claridad a partir de código legible o de nombres + `depends`,
  pero NO está contrastado con datos. **Solo si lo ves claro.** Etiquétalo siempre como aproximación.
- **No determinable** — ni se puede leer ni se ha podido observar con datos.

Observar un valor en N registros es un hecho; **inferir la fórmula que lo produjo es una suposición**
→ prohibido presentarla como cierta. Una relación que se cumple en la muestra podría fallar en un
caso extremo no muestreado: por eso "comportamiento observado" ≠ "confirmado". En la duda, baja de nivel.

## Líneas rojas de los entregables de cliente

- **Nunca** menciones al proveedor/partner anterior ni al "código fuente original" (no lo obtendrán).
- Audiencia = usuarios de Odoo **sin conocimientos de programación**: lenguaje funcional, sin jerga
  de código. Habla de "vistas e informes", "automatismos", "cálculos", no de clases ni métodos.
- No inventes ni deduzcas formulas internas. Lo que no es seguro se marca como aproximación o como
  no determinable, y no pasa nada por dejarlo así.

## Refinamientos de fiabilidad (lo que hace falso un informe si se ignora)

- **Aislar el delta custom.** Observar entrada→salida ve *estándar + custom* mezclados. Para modelar
  solo lo a medida, regresa sobre la **diferencia** cliente−vanilla (Canal D), no sobre la salida bruta.
- **Overrides sin footprint.** Redefinir `action_confirm`/`_prepare_invoice`/`create`… no deja rastro
  en metadatos. Cázalo con el estático del `.pyc` o con el diff de comportamiento.
- **Palimpsesto de versiones.** Los datos acumulan varias generaciones de código; segmenta por
  `write_date`. Campos `store` pueden guardar valores de código ya inexistente.
- **No-determinismo.** Cálculos que dependen de fecha/usuario/secuencia/estado externo no son función
  pura de sus inputs → no se les infiere fórmula; se marcan "depende de contexto".
- **Punto ciego `@api.onchange`.** Lógica solo de UI: invisible a los datos; solo estático o UI en sandbox.
- **Muestreo.** `active_test=False` + multicompañía + `sudo` (en sandbox) o el número de fiabilidad
  está inflado por record rules y archivados.

## Las fases del flujo

1. **Conexión y seguridad** (`references/connection.md`).
2. **Descubrir módulos a medida** → `discover_modules.py`. Acordar la lista con la persona.
3. **(Opcional) Mapear áreas** → crear `<out_dir>/areas.json` (ver `assets/areas.example.json`)
   para agrupar módulos por área funcional (RRHH, Contabilidad, Ventas…). Si no, cada módulo es su
   propia área.
4. **Extracción estructural** → `extract_module.py <out_dir> <mod1> <mod2> ...`.
5. **Lógica visible** → `extract_logic.py <out_dir> <mod1> ...` (dependencias reales con `fields_get`).
6. **Uso real** → `count_records.py <out_dir>`.
7. **Consolidar hechos** → `build_facts.py <out_dir>` (clasifica nuevo/extensión, modelos distintos).
7b. **Análisis de comportamiento (caja negra)** → `analyze_behavior.py <out_dir>` (muestrea registros
   reales y guarda evidencia entrada→salida de los campos calculados; read-only).
8. **Análisis por módulo (un agente por módulo)** → ver `references/per-module-analysis.md`.
9. **Consolidación ponderada** → `consolidate.py <out_dir>`.
10. **Informe** → `build_report.py <out_dir>` y renderizar a PDF con la skill `pdf-renderer`.

## El canal ir.logging (opcional)

`ir.logging` es la tabla técnica de logs de Odoo. Tiene dos usos en ingeniería inversa, y conviene
no confundirlos:

- **Pasivo (recomendado, read-only):** si el código a medida hace `_logger.info/debug(...)` en sus
  métodos, al ejecutarse deja rastros ahí que revelan **comportamiento en tiempo real** (qué función
  corre, desde qué fichero, con qué mensaje). Léelos con `scripts/read_logging.py <out_dir>`. Es una
  pista muy valiosa **cuando el desarrollo loguea**. Si no loguea (muchos no lo hacen), el script te
  lo dice y no pasa nada: te apoyas en los metadatos y en el código legible de automatismos, que es
  lo que reconstruye el sistema en la mayoría de casos.
- **Activo (opt-in, requiere OK del cliente):** escribir nosotros en `ir.logging` como cuaderno de
  captura. Es inocuo (tabla técnica, borrable, sin impacto en negocio), pero **escribir no revela la
  lógica compilada por sí mismo**; solo sirve para dejar trazas/notas propias. No forma parte del
  flujo read-only por defecto y `read_logging.py` no escribe.

Idea clave para gestionar expectativas: `ir.logging` **no es la fuente principal** de la
reconstrucción salvo que el código del cliente loguee mucho. Lo que casi siempre reconstruye el
sistema son los metadatos (`ir.model`, `fields_get`) y el **código en texto plano** de acciones de
servidor y crons. Usa `ir.logging` como complemento, no como vía única.

**Ojo con "simular los flujos":** ejecutar un flujo solo enciende `ir.logging` **si ese código tiene
llamadas `_logger`**. Si el desarrollo no loguea (lo habitual en código compilado), por mucho que
simules los flujos `ir.logging` seguirá en silencio. Para subir el % en ese caso la vía NO es
`ir.logging`, sino **observar los datos que el flujo produce**: en read-only con `analyze_behavior.py`
(muestreo entrada→salida), y de forma plena ejecutando el proceso en preproducción y comparando el
**antes/después de los datos** (con las integraciones externas —AEAT, SERES, etc.— aisladas, y con
autorización del cliente, porque eso ya no es solo lectura).

## Por qué ponderar por uso

Un módulo con 500 campos pero 0 registros importa menos que uno con 50 campos y 300.000 registros.
El conocimiento global es la media de `conocimiento_pct` por módulo **ponderada por nº de registros**,
para que el número refleje lo que de verdad mueve datos. Cuidado al sumar modelos: un modelo estándar
ampliado por varios módulos se contaría varias veces, por eso los totales globales usan modelos
**distintos** (lo hace `build_facts.py`).
