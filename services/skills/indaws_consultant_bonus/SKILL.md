---
name: indaws-consultant-bonus
description: >-
  Calcula el bonus/variable trimestral del equipo de consultoría de Dynapps Spain (ex-Indaws) a
  partir de datos reales de Odoo. Usar cuando se hable de bonus, variable, objetivos de horas,
  cumplimiento de objetivos, liquidación trimestral, 'como voy de objetivo', 'cuanto bonus me
  llevo', objetivos de equipo Tokyo/Monaco/Servicios, o informe de bonus para Dirección/COO/PM,
  aunque no se diga la palabra 'bonus'. Dos modos: respuesta rápida para el consultor e informe HTML
  completo (marca Dynapps Spain) para Dirección.
version: 1.1.2
effort: high
model: opus
---

# Consultant Bonus — Bonus trimestral de consultoria

Calcula el variable (20% del sueldo) del equipo de consultoria de **Dynapps Spain**
(equipos Tokyo / Monaco / Servicios) leyendo datos reales de Odoo en SOLO LECTURA.

**Dos modos:**
1. **CONSULTOR** (dia a dia): respuesta breve en chat con % de objetivo, tramo y % de variable resultante; no genera documento.
2. **DIRECCION** (COO/CEO/PM): informe HTML completo con marca Dynapps Spain.

> Regla de oro: NUNCA calcular a mano. Trae datos por MCP → estructura según `references/reglas-bonus.md` §9 → ejecuta `scripts/calculate_bonus.py`. Salida SIEMPRE en **% del sueldo** (sin EUR).

> **Periodos anteriores a septiembre de 2026** se liquidaron con la fuente vieja (campos de
> `crm.lead`), que contaba menos demos. AVISA antes de calcular que la cifra no coincidirá con lo
> pagado. Qué hacer (recalcular, abonar o dejarlo) lo decide Dirección; la skill solo advierte.

## Workflow

1. **Paso 0 — Conexion MCP**: Identifica el MCP de Odoo de Dynapps Spain. `whoami` para posicionar a la persona. Lee `references/data-model.yaml`. NUNCA `orm_create/write/unlink`. NUNCA pasar `context`.
2. **Paso 1 — Modo**: CONSULTOR (pregunta personal, 1 persona) vs DIRECCION (informe, todos los equipos, COO/PM). Autoselecciona según `whoami`.
3. **Paso 2 — Periodo**: Trimestre (cerrado o en curso → previsión) o año completo. Objetivos por defecto: NO preguntar, MOSTRAR en Supuestos.
4. **Paso 3 — Validacion previa** (OBLIGATORIA, STOP): Mostrar y confirmar: (1) despidos/salidas con fecha aprox, (2) incorporaciones recientes + umbral de cobro, (3) perfiles mixtos/overrides por periodo, (4) cambios de equipo con fechas, (5) tareas `[reparto]` candidatas. NO calcular nada hasta confirmación.
5. **Paso 4 — Extraccion** (MCP, solo lectura): Población con `["active","in",[true,false]]` (incl. archivados). `read_group` sobre `account.analytic.line` agrupando SOLO por `employee_id` (`lazy:false`). Demos: `mail.activity` (tipo "Demo" resuelto por nombre) con `["active","=",false]` y `date_done` en el periodo → **agrupar por `res_id`** antes de contar. Paginar con `offset` (max 500). Ver `references/data-model.yaml`.
6. **Paso 5 — Calcular**: `echo '<json>' | python3 scripts/calculate_bonus.py`. NUNCA a mano.
7. **Paso 6 — Salida**: CONSULTOR → respuesta breve en chat. DIRECCION → `python3 scripts/fill_report.py` con `datos_informe.json`. Guardar en `output/`.

## GOTCHAS

1. **Nunca `context` al MCP** — lo rechaza con "Internal server error". El idioma ya sale en es_ES por defecto.
2. **read_group UN solo nivel** — NUNCA groupby por proyecto/tarea (explota). Penalización 0.7 = `unit_amount − x_studio_bonus_unit_amount`. El campo ya lo trae calculado — no reaplicar.
3. **Incluir ARCHIVADOS** — `["active","in",[true,false]]` en `hr.employee` y `res.users`.
3b. **Demos = `mail.activity` CERRADAS, y las cerradas están ARCHIVADAS** — los campos de demo de `crm.lead` fueron BORRADOS (devuelven "Invalid field"). Una demo hecha es una actividad con `active=False` y fecha `date_done` (NO `date_deadline`). El dominio DEBE llevar `["active","=",false]` explícito: es lo que desactiva el filtro implícito de Odoo. Sin él la consulta devuelve **CERO** — verificado a propósito en Q2 2026: 96 actividades con el filtro, 0 sin él. El tipo "Demo" se resuelve POR NOMBRE (38 en producción, cambia entre instancias).
3c. **UNA demo por OPORTUNIDAD** — agrupar por `res_id` ANTES de contar. Un lead puede tener varias actividades Demo; contarlas todas infla el bonus. Q2 2026: 96 actividades pero **79 oportunidades** → 79 × 4 = **316 h**. Si sale 96, se están contando actividades.
4. **Roles cambian entre trimestres** — detectar el vigente en el periodo; overrides son por periodo y confirmables, nunca permanentes.
5. **Denominador de equipo = solo establecidos** — `cuenta_bonus=true` incl. devs. Becarios/incorporaciones: horas al numerador pero NO al denominador. Ranking 02: solo quien `cobra=true`.
6. **Validacion previa es OBLIGATORIA** — NO calcular nada sin confirmación del usuario. Despidos/fechas son aproximaciones: mostrar, no asumir.
7. **Solo 3 equipos**: Tokyo 28, Servicios 32, Monaco 33. Las 145h del PDF son jornada, NO el objetivo de bonus.

Ver `references/gotchas-extended.md` para detalles operativos completos.

## Sanity gates (antes de entregar)

Invariantes de `references/reglas-bonus.md` (§1,§5,§6,§7) y `casos-especiales.md`. Comprobar sobre la salida del script; fuera de banda → STOP, NO mostrar.

- [ ] **Topes por rol respetados**: consultor individual<=0.14 / equipo<=0.06; PM individual<=0.08 / equipo<=0.12; desarrollo no cobra (var=0). `var_total<=0.20` SIEMPRE → fuera de banda → revisa config/datos, no entregues.
- [ ] **Objetivos correctos por rol**: individual 113 consultor / 80 PM; equipo 105 consultor / 85 dev / 80 PM (×meses). Las 145h son jornada, NO objetivo.
- [ ] **Tramos solo {0.0, 0.5, 1.0}** (escala sin huecos: >=1.00→1.0, 0.80-1.00→0.5, <0.80→0.0) → cualquier otro valor → bug, STOP.
- [ ] **Prorrateo por días aplicado** (`frac_activa` 0..1) en bajas/incorporaciones/cambios de equipo → confirmado en Paso 3.
- [ ] **Sin división por cero**: `obj_equipo>0` (denominador = solo establecidos `cuenta_bonus=true`) → si 0 → revisa población, no calcules pct.
- [ ] **`var_en_juego_pct` en 0-100** → fuera de rango → bug del cálculo, STOP.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| dynapps-brand | common | Marca del informe HTML (colores, Readex Pro, logo, "Dynapps Spain") |

## Consumed by

Ninguno — skill de uso directo (consultor / Direccion).

## References

| Archivo | Cuando leer |
|---------|-------------|
| `references/data-model.yaml` | **Antes de cualquier consulta** — donde vive cada dato en Odoo |
| `references/reglas-bonus.md` | **Antes de calcular** — objetivos, tramos, splits, formulas, contrato E/S del script |
| `references/casos-especiales.md` | Demos, despidos, cambios de equipo, incorporaciones, Aitor, Angel, [REPARTO] |
| `references/alertas.md` | Para generar las alertas por empleado (incl. vacaciones) |
| `references/gotchas-extended.md` | Detalles operativos completos de todos los GOTCHAS |
| `references/golden-examples.md` | Ancla anti-drift: 2 casos input→bonus congelados. Confirmar antes de tocar reglas |
| `templates/informe_direccion.html` | Modo DIRECCION — plantilla del informe |
| `references/shared/odoo-connection.md` | Connection pattern y autenticación Odoo vía MCP |
