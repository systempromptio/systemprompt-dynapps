---
name: indaws-hr-performance
description: >
  Gestiona el ciclo de evaluaciones de desempeno en Indaws. Genera documentos de
  preparacion de review para managers, guias de autoevaluacion, vista de calibracion
  de equipo (9-box grid), plantillas de PIP con acciones SMART, dashboard de estado
  del ciclo de reviews y analisis de career ladder / promotion readiness.
  Usar cuando el usuario pida: performance review, evaluacion de desempeno,
  review prep, autoevaluacion, self-assessment, calibracion, 9-box, PIP, plan de
  mejora, career ladder, promocion, objetivos, goals, appraisal.
version: 1.1.2
model: sonnet
effort: medium
---

# HR Performance Review Management

**CONFIDENTIAL - HR TEAM ONLY**
**Document ID:** HR-SKILL-003

Ciclo completo de evaluaciones de desempeno. Conecta con HiBob (Goals) y Odoo (hr.appraisal)
vía la librería compartida `../../lib/` para generar review prep, autoevaluaciones, calibracion
9-box, PIPs, cycle status y promotion readiness. Datos altamente sensibles (RGPD/legal).

---

## Prerequisitos

- **HiBob API** + **Odoo XML-RPC**: credenciales en entorno (ver `lib/config.py`).
- **Python 3** para los scripts; **librería compartida** en `../../lib/`.
- **RGPD**: leer `../../security/gdpr_guidelines.md` antes de mostrar cualquier dato.

---

## Workflow

### Paso 0: Recopilacion (DETENTE — esperar respuesta)
1. **Operacion**: `review-prep` | `self-assessment` | `calibration` (9-box) | `pip` |
   `cycle-status` | `promotion-readiness`.
2. **Alcance**: empleado / departamento / empresa (esta ultima solo para `cycle-status`).
3. Según operacion: review-prep → empleado, manager, periodo, tipo; pip → empleado, areas,
   duracion (def. 90d), acciones previas; promotion → empleado, nivel actual y nivel objetivo.

### Paso 1: Extraccion de datos
```bash
python3 scripts/fetch_performance_data.py --output data.json \
  [--employee "<nombre>" | --department "<dpto>"] [--months 12]
```
Extrae HiBob Goals (progreso/estado), Odoo hr.appraisal (historico), HiBob People (antiguedad),
feedback y KPIs. Verificar que el empleado existe en AMBOS sistemas y que hay objetivos del
periodo; si faltan datos, informar antes de continuar.

### Paso 2: Calculo (OBLIGATORIO - nunca manual)
```bash
python3 scripts/review_generator.py --mode <review-prep|calibration|pip|promotion> \
  --input data.json --framework references/review_framework.md \
  --competencies references/competency_model.md \
  --career-ladder references/career_ladder.md [--duration 90] [--target-level "<nivel>"]
```
El script aplica framework + competencias y calcula: progreso de objetivos, puntuacion por
competencia, tendencia vs appraisals previos, posicion 9-box, gap de promocion, acciones SMART de
PIP y estado del ciclo. Detalle de matrices/scorecards: `references/frameworks.md`.

### Paso 2.5: Validacion (DETENTE — esperar aprobacion)
PRESENTAR resumen del resultado (objetivos y %, competencias, tendencia / cuadrantes 9-box /
hitos PIP / gap). El usuario puede pedir ajustes.

### Paso 3: Generacion del informe
SOLO tras aprobacion. LEER la plantilla y generar el documento, GUARDAR en el directorio de trabajo:
- Review prep → `templates/review_prep.md` (estructura en `references/frameworks.md` §2)
- Self-assessment → `templates/self_assessment.md` (§3)
- Calibration view → tabla 9-box (§1)
- PIP → `templates/pip_template.md` (§4)
- Cycle status → tabla (§5) · Promotion readiness → gap analysis (§6)

---

## GOTCHAS

- **9-box no ubicable**: la calibracion necesita rating de Rendimiento Y de Potencial. Si falta
  uno, el empleado no se puede situar en la matriz → DETENERSE y pedir el dato.
- **PIP = dato critico RGPD/legal**: nunca circula sin revision de RRHH y asesor laboral; jamas
  fuera del circuito manager+HR.
- **Self-assessment vs manager review desalineados**: son perspectivas independientes; una
  divergencia fuerte (5 vs 2) se marca para conversacion, NO se promedia en silencio.
- **Career ladder por familia de puesto**: un IC4 de ingenieria != IC4 funcional; usar el track
  correcto al evaluar readiness o el gap analysis sale erroneo.
- **Fuente de verdad**: HiBob es primaria para objetivos; Odoo (hr.appraisal) para appraisals
  historicos. No mezclar ni dar por buena una sola fuente cuando la otra contradice.
- **Datos insuficientes**: empleado sin objetivos del periodo o sin appraisals previos → avisar
  antes de generar, no rellenar con supuestos.

---

## NEVER

- NUNCA generar evaluaciones, puntuaciones o recomendaciones manualmente: siempre `review_generator.py`.
- NUNCA generar un documento sin la aprobacion del Paso 2.5.
- NUNCA exponer datos de desempeno o un PIP fuera del circuito manager + RRHH.
- NUNCA compartir la evaluacion de un empleado con otro empleado.
- NUNCA incluir datos sensibles (salario, salud, NIF/NIE) en documentos de performance (RGPD).
- NUNCA inventar datos: todo procede de HiBob y Odoo.

## ALWAYS

- SIEMPRE tratar desempeno y PIP como altamente confidenciales (manager + HR).
- SIEMPRE usar la escala 1-5 (`references/review_framework.md`) y cruzar autoevaluacion + vision del manager.
- SIEMPRE revisar los PIP con RRHH y asesor laboral antes de entregarlos.

---

## Delegates to
- **hr-employee-lookup** — ficha/datos básicos del empleado (no re-consultar HiBob People aquí).

## Consumed by
- **hr-people-report** — agrega estado del ciclo y señales de talento al informe mensual.
- Managers / RRHH que preparan reviews, calibraciones, PIPs y promociones.

---

## REFERENCES

- `references/review_framework.md` — ciclos, proceso, escala 1-5; `competency_model.md` — competencias por nivel.
- `references/career_ladder.md` — niveles IC1-IC6 / M1-M3 por familia de puesto.
- `references/frameworks.md` — matriz 9-box, scorecards, plantilla PIP, dashboard, umbrales.
- `templates/` — `review_prep.md`, `self_assessment.md`, `pip_template.md`.
- `../../lib/` — `hibob_client.py`, `odoo_hr_client.py`, `config.py`, `report_utils.py`.
- `../../security/gdpr_guidelines.md` — enmascaramiento RGPD obligatorio.
- `common:references/shared/` — N/A: HiBob/Odoo vía `lib/` propio de la suite.

---

## Success Criteria

- [ ] Datos extraidos de HiBob y Odoo; `review_generator.py` ejecutado sin errores.
- [ ] Resumen presentado y aprobado (Paso 2.5).
- [ ] Objetivos y competencias evaluados con datos reales segun framework y nivel.
- [ ] 9-box con ambos ejes; PIP con acciones SMART; promotion con gap analysis y evidencias.
- [ ] Ningun dato sensible expuesto (RGPD); documento guardado.
