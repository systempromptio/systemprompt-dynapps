# GOTCHAS extendidos — Consultant Bonus

Complementa los GOTCHAS del SKILL.md con detalles operativos críticos.

## Datos y campos

- **El campo bonus ya trae el 0.7** — `x_studio_bonus_unit_amount` ya está ponderado. NUNCA reaplicar el factor.
- **Facturabilidad propia Indaws** — usa `x_studio_bonus_unit_amount`/`billable_unit_amount`, NO `timesheet_invoice_type` estándar.
- **Equipo del EMPLEADO** (`hr.employee.x_studio_team_id`), no el del proyecto — salvo en el caso Angel.
- **[REPARTO] ya está en el total** — solo se analiza, NO se suma aparte (evita doble conteo).
- **write_date y create_date son aproximaciones** — SIEMPRE mostrar y confirmar antes de calcular.
- **Solo 3 equipos**: Tokyo 28, Servicios 32, Monaco 33. Paris y demás fuera.
- **Las 145h del PDF son jornada para 'facturabilidad', NO el objetivo** — no las uses para el bonus.

## Queries MCP

- **read_group disciplinado** — un solo nivel de groupby, `lazy:false`, solo `:sum`. Penalización 0.7 = `unit_amount − x_studio_bonus_unit_amount` por empleado. NUNCA groupby por proyecto/tarea (explota a cientos de miles de filas).
- **NUNCA pasar `context`** — el MCP lo rechaza ("Internal server error"). Archivados via dominio, no `active_test`.
- **Solo lectura** — jamás `orm_create/write/unlink`. Este skill no escribe en Odoo.

## Población y roles

- **Incluir ARCHIVADOS (C1)** — población con `["active","in",[true,false]]` + unión con quien imputó en el periodo. NUNCA asumir un nº fijo de personas.
- **Roles cambian entre trimestres** — detectar el rol vigente en el periodo; overrides (Aitor 80/80) son por periodo y confirmables, nunca permanentes.
- **Denominador de equipo = SOLO establecidos** — Σ objetivo por rol × frac de quien cobra (`cuenta_bonus=true`, incl. devs). Becarios/incorporaciones (`cuenta_bonus=false`): horas al numerador, objetivo NO al denominador, y NO aparecen en el ranking 02.
- **`cuenta_bonus` es SOLO flag de antigüedad** — un desarrollador establecido lleva `cuenta_bonus=true` aunque no cobre. El motor calcula `cobra = cuenta_bonus AND rol≠desarrollo`. No marques a los devs como `cuenta_bonus=false`.

## Informe Dirección

- **Ranking 02 = solo quien cobra** — fuera desarrollo y `cuenta_bonus=false`; cuentan para el equipo pero no salen en el ranking.
- **Altas/bajas a mitad de trimestre**: mostrar fecha y % en el ranking, p.ej. "Claudia Martínez · alta 15-02 · cuenta 50%".
- **Columna "% variable"** = `var_en_juego_pct` (0-100%), NO la fracción de sueldo.
- **Columnas "% indiv./equipo"** del ranking = logro·tramo (p.ej. "90% · 50%"). El % real se ve en el detalle por equipo.
- **Misma escala individual y equipo** — ≥100→100% / 80-99→50% / <80→0%.
- **Demos = actividades CERRADAS, y las cerradas están ARCHIVADAS** — las demos ya no se leen de `crm.lead` (campos borrados) sino de `mail.activity`. Una demo hecha tiene `active=False`, así que sin ver las archivadas la consulta devuelve **CERO**. Verificado en rojo, a propósito, sobre Q2 2026: **96 actividades con el filtro, 0 sin él**. El `["active","=",false]` explícito del dominio es lo que desactiva el filtro implícito de Odoo — no es opcional. La fecha es `date_done`, no `date_deadline`.
- **Demos: UNA por OPORTUNIDAD** — agrupar por `res_id` antes de contar. Un lead puede tener varias actividades Demo y contarlas todas infla el bonus. Q2 2026: 96 actividades pero **79 oportunidades** (13 con más de una). El número que cuenta es 79 → 316 h.
- **% del sueldo, no EUR** — salvo que el usuario aporte sueldos.
