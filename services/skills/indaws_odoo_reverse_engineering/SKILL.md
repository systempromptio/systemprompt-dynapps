---
name: indaws-odoo-reverse-engineering
version: 1.2.4
effort: high
model: opus
description: >-
  Ingenieria inversa read-only, via API, del codigo a medida de un Odoo de cliente cuando no hay fuente: que hace cada modulo, uso real e informe para el cliente. Úsala con código compilado/cerrado de otro partner o un Odoo heredado sin fuente, aunque no digan «ingeniería inversa». NOT auditoría con fuente (→ odoo-code-auditor); NOT informe de estado técnico (→ odoo-rescue-pack).

---

# Ingeniería inversa de Odoo (READ-ONLY)

Reconstruye y documenta el sistema a medida de un cliente Odoo cuyo código está **compilado**
(`.pyc`, no legible), produciendo un informe entendible por el cliente. Contexto típico: un cliente
migra a Indaws desde otro partner y hay que entender qué hacen sus desarrollos sin código fuente.

**Dos modos según el acceso:** el **modo API** (por defecto) es solo lectura contra producción, pasos
1-10. Con un **backup restaurable**, el **modo backup** añade filesystem (`.pyc`), sandbox, inferencia
de fórmula y diff contra un Odoo limpio (pasos **(B)**; `references/backup-restored-mode.md`).

## Antes de empezar — tres reglas

1. **Solo lectura contra el sistema del cliente.** En **modo API** trabajas sobre producción con
   gente dentro: `scripts/_odoo.py` bloquea todo lo que no sea lectura y nunca mutas datos **a
   propósito**. No prometas más: leer ejecuta el código a medida del cliente, y ese código puede
   escribir (gotcha 5). La **única excepción** deliberada es el sandbox del **modo backup**: ahí sí
   se escribe, pero sobre **copia desechable, jamás producción**, con las integraciones cortadas y
   OK explícito del cliente.
2. **Certeza o nada.** Reporta en niveles: *confirmado* (100 %), *aproximación* (solo si se ve
   claro, etiquetada como tal) y *no determinable*. Observar un valor es un hecho; inferir la fórmula
   que lo produjo es una suposición → no la presentes como cierta.
3. **Entregables de cliente.** Nunca menciones al proveedor anterior ni el "código fuente original".
   Escribe para usuarios de Odoo **sin conocimientos de programación**.

Qué se puede leer y qué no, y el porqué, en `references/methodology.md` — léelo al arrancar.

## El flujo, paso a paso

Trabaja en un directorio de salida (`<out_dir>`, p. ej. `<proyecto>/_re/`). Los scripts (`scripts/`) leen la conexión de variables de entorno.

**1. Conexión.** Pide credenciales y expórtalas (`ODOO_URL`, `ODOO_DB`, `ODOO_USER`, `ODOO_KEY`).
Confirma que es producción y que vas en solo lectura. Contrato de conexión y seguridad read-only:
`references/shared/odoo-connection.md` + `references/connection.md` (protocolo por versión).

**2. Descubrir los módulos a medida.** `python3 scripts/discover_modules.py <out_dir> [--author "Autor"]`.
Revisa `<out_dir>/modules.json` **con la persona**: acordar esa lista condiciona todo lo demás.

**3. (Opcional) Mapear áreas funcionales.** Para agrupar por área (RRHH, Contabilidad…), crea
`<out_dir>/areas.json` (formato en `assets/areas.example.json`); si no, cada módulo es su área.
Debe existir **ANTES** de `build_facts.py` (paso 7); si lo creas después, relanza 7 y 9.

**4. Extracción estructural** de cada módulo acordado:
`python3 scripts/extract_module.py <out_dir> <mod1> <mod2> ...`. Genera `<mod>.json` (crudo, incluido
el código de acciones de servidor/crons y el arch de vistas) y `<mod>.md` (informe legible).

**5. Lógica visible** (deps reales de campos calculados vía `fields_get`):
`python3 scripts/extract_logic.py <out_dir> <mod1> <mod2> ...`

**6. Uso real** (registros por modelo, `search_count`): `python3 scripts/count_records.py <out_dir>`

**6b. (Opcional) Canal ir.logging.** `python3 scripts/read_logging.py <out_dir>` si el código deja
rastros en `ir.logging` (si no loguea, el script lo dice y sigues con metadatos). Detalle en `references/methodology.md`.

**7. Consolidar hechos** (clasifica nuevo/extensión, divide campos, calcula modelos distintos):
`python3 scripts/build_facts.py <out_dir>`.

**7b. Análisis de comportamiento (caja negra, READ-ONLY).** Para cada campo calculado a medida,
muestrea registros reales y guarda la evidencia entrada→salida:
`python3 scripts/analyze_behavior.py <out_dir> --samples 200`. Sube hallazgos de "no determinable" a
"comportamiento observado" cuando los datos reales lo confirman sin excepciones. **Antes de lanzarlo
contra producción**: muestrear un calculado **ejecuta el `compute` a medida** (gotcha 5) — pregunta
si alguno tiene integración externa detrás, porque un webservice se dispararía 200 veces y eso no lo
revierte nadie. `--only-stored` limita el muestreo a campos que se leen de la columna.

**7c. Cobertura extra (siempre, read-only).** Informes QWeb (a menudo *son* el desarrollo),
automatizaciones, integraciones (`ir.config_parameter`, crons), footprint de `ir.model.data` y
seguridad: `python3 scripts/extract_extras.py <out_dir>` → `<mod>.extras.json`.

**(B) Solo en modo backup restaurado** — aditivo, con filesystem/BD desechable. Cuatro canales que
suben la certeza: estático del `.pyc` (`static_pyc.py`), inferencia de fórmula (`infer_formula.py`),
diff contra Odoo limpio (`dump_fields.py` + `baseline_diff.py`) y sandbox instrumentado. Comandos y
reglas en `references/backup-restored-mode.md` — **léelo antes de tocar el modo backup**.

**8. Análisis por módulo — un agente por módulo** `[Agent(model: sonnet)]`. Lee
`references/per-module-analysis.md` y lanza el fan-out (`Workflow` si la tienes): cada agente analiza
su módulo desde los ficheros ya extraídos y escribe `<out_dir>/<mod>.analysis.json` (descripción
funcional, modelos críticos, lógica en 4 niveles, % conocimiento). Valida que estén todos.

**9. Consolidación ponderada por uso:** `python3 scripts/consolidate.py <out_dir>`. Produce
`<out_dir>/consolidated.json` y el % global ponderado por registros.

**10. Informe.** (Opcional) `<out_dir>/report_meta.json` (`assets/report_meta.example.json`): elige
**marca** (`indaws` por defecto o `dynapps`) y secciones opcionales — ver
`references/report-structure.md`. Luego `python3 scripts/build_report.py <out_dir>` genera
`informe-ingenieria-inversa.html`; renderízalo con **`pdf-renderer`** y **revísalo página a página**
(`pdftoppm` a imágenes) antes de entregar.

## GOTCHAS

1. **`.pyc` no legible**: reconstruyes por comportamiento observado (caja negra), no por lectura de
   fuente → nunca presentes una fórmula inferida como confirmada (regla de certeza de 3 niveles).
2. **`areas.json` tarde**: si lo creas después del paso 7, `build_facts.py`/`consolidate.py` ya
   agruparon mal → relánzalos.
3. **Modelos abstractos/sin tabla/sin acceso** fallan en extracción: no es error grave, no tienen
   registros y se omiten. Sigue con el resto.
4. **No infles totales**: un modelo estándar ampliado por varios módulos se cuenta **una vez** en los
   totales globales (lo gestiona `build_facts.py`); recuérdalo si reportas a mano.
5. **«Read-only» son dos cosas y solo una es cierta** (medido v18/v19, detalle y tabla en
   `references/methodology.md` §«Solo lectura»): el wrapper **sí** bloquea todo lo que no sea uno de
   sus diez métodos de lectura, pero esos diez **ejecutan código del cliente** —el `compute` de un
   campo leído, el `_search_…` de un dominio, el `_compute_display_name` de un `name_search`— y si
   ese código escribe, escribe. Sin código a medida de por medio: **leer un campo almacenado
   pendiente de recalcular lo persiste**. Antes del paso 7b sobre producción, pregunta si algún
   campo a medida tiene integración externa detrás: eso no lo revierte nadie.
6. **Diagnóstico, no migración**: la lógica no determinable se valida luego con pruebas funcionales
   durante la migración; déjalo dicho en la recomendación del informe.
7. **Modo backup — trampas que hacen falso un informe** (overrides sin footprint, palimpsesto de
   versiones, no-determinismo, punto ciego `@api.onchange`, muestreo sesgado): detalladas en
   `references/backup-restored-mode.md` y `references/methodology.md`. Léelas antes de reportar fiabilidad.

## NEVER / ALWAYS

- **NEVER** escribir ni mutar datos **en el sistema del cliente / producción** (la única ejecución con
  escritura permitida es el sandbox del modo backup, sobre copia desechable y con OK del cliente);
  **NEVER** mencionar al partner anterior o el "código fuente original" en el entregable; **NEVER**
  presentar una inferencia (fórmula recuperada, decompilado) como hecho confirmado.
- **ALWAYS** confirmar entorno + alcance antes de conectar (producción→solo lectura; sandbox→copia
  aislada con integraciones cortadas); acordar la lista de módulos con la persona (paso 2); revisar el
  PDF página a página antes de entregar.

## Referencias (léelas cuando toque)

- `references/connection.md` — credenciales, seguridad read-only, protocolo por versión, identificar custom.
- `references/methodology.md` — qué se lee y qué no, regla de certeza de 3 niveles, líneas rojas, fases.
- `references/per-module-analysis.md` — cómo lanzar el agente por módulo + prompt y esquema JSON exactos.
- `references/report-structure.md` — métricas, secciones, render · `references/backup-restored-mode.md` — modo backup: `.pyc`, sandbox, inferencia, diff vs limpio.

## Delegates to
- `pdf-renderer` (render HTML→PDF) · `dynapps-brand` (marca del PDF).

## Consumed by
- `odoo-rescue-pack` (motor sin fuente, casos 2 y 3; devuelve el `<out_dir>` que integra) · pre-migración/adopción desde otro partner · standalone.
