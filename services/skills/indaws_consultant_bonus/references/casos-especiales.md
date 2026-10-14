# Casos especiales — consultant-bonus

Todos estos casos se DETECTAN y se MUESTRAN al usuario para que CONFIRME **antes**
de calcular nada. El skill nunca asume: propone valores por defecto y espera el OK.

## 1. Demos (+4h)

- Fuente: **`mail.activity`**, no `crm.lead`. Los campos de Studio que se usaban antes fueron
  BORRADOS de producción; mencionarlos devuelve "Invalid field".
- **Tipo de actividad: resolver POR NOMBRE**, nunca hardcodear el id — `mail.activity.type` con
  `name='Demo'` y `res_model='crm.lead'`. En producción es el 38, pero cambia entre instancias.
- Dominio:
  `[["activity_type_id","=",<tipo>],["res_model","=","crm.lead"],["active","=",false],["date_done",">=",<inicio>],["date_done","<=",<fin>]]`
- **Una demo HECHA es una actividad CERRADA** (`active=False`). La fecha es **`date_done`**, NO
  `date_deadline`: una demo agendada y no hecha NO cuenta.
- **Las cerradas están ARCHIVADAS.** Sin verlas la consulta devuelve **CERO** — medido en Q2 2026:
  96 actividades con el filtro, 0 sin él. El `["active","=",false]` del dominio es lo que desactiva
  el filtro implícito de Odoo y es OBLIGATORIO. Si el cliente admite contexto, añadir además
  `{"active_test": false}`; por MCP no se puede (GOTCHA 1).
- El resultado de la oportunidad (ganada/perdida/congelada) **ya no interviene**: se consultan
  actividades, no leads, así que no hace falta ningún filtro sobre el `active` del lead.

### ⭐ Regla de conteo: UNA demo por OPORTUNIDAD como máximo

Antes era automático, porque el lead tenía un único campo de fecha. Ahora **un lead puede tener
varias actividades Demo**, y contarlas todas inflaría el bonus.

1. Traer las actividades Demo cerradas con `date_done` dentro del periodo.
2. **AGRUPAR POR `res_id`** (la oportunidad).
3. Cada grupo = **UNA** demo, valga lo que valga su número de actividades.
4. Participantes = conjunto de `user_id` **DISTINTOS** de las actividades de ese grupo.
5. **+4h ÷ nº de participantes**. Si 1 participante → +4h; si 2 → +2h c/u. Planas (sin 0.7).
6. Se imputan al **equipo del empleado** (`hr.employee.x_studio_team_id`), no al `team_id` del lead.
   Mapear cada participante res.users → `hr.employee.user_id` → su equipo.

**Contraste de Q2 2026** (para comprobar la implementación):

| | |
|---|---|
| Actividades Demo cerradas | 96 |
| **Oportunidades distintas** | **79** ← es el número que cuenta |
| Oportunidades con más de una actividad | 13 |
| Participantes por oportunidad | 69 con 1 · 8 con 2 · 2 con 3 |
| Horas de bonus | 79 × 4 = **316 h** |

> Si el cálculo da **96**, se están contando ACTIVIDADES en vez de OPORTUNIDADES.

## 2. Despidos / salidas

- Fuente: `res.users` con `active=False` y `share=False` (context `active_test=false`), orden `write_date desc`.
- `write_date` = fecha aproximada de baja (ÚNICO criterio; `departure_date` NO se usa, no se rellena).
- Cruce a empleado por `employee_id`; si está roto (pasa al archivar), cruzar por `login`/email.
- Excluir externos (dominios ajenos, OdooBot, etc.).
- **OBLIGATORIO**: mostrar tabla [usuario · fecha write_date · empleado] y pedir confirmación de la fecha de baja antes de prorratear.
- Prorrateo: `meses_activos` = meses del periodo hasta la baja (ver reglas-bonus §7).

## 3. Cambios de equipo

- No existe fecha de cambio en Odoo → **preguntar al usuario** empleados, fechas y equipos.
- Las horas y el objetivo se reparten entre equipos según los meses en cada uno.
- Ejemplo: 1 mes en Tokyo + 2 en Mónaco → 1/3 a Tokyo, 2/3 a Mónaco (objetivo y horas).

## 4. Nuevas incorporaciones

- Fuente de alta: `hr.employee.create_date` (ÚNICO criterio).
- Umbral según `job_id.name`: **1 mes** si con experiencia (contiene "senior"/"medior"),
  **6 meses** si junior (contiene "junior"/"becario"/"prácticas").
- Bajo el umbral: `cuenta_bonus=false` → NO percibe bonus, y **NO aparece en el ranking 02**.
  **Sus horas SÍ suman al numerador** del equipo, pero su **objetivo NO entra en el
  denominador** (solo cuentan los establecidos; ver reglas-bonus §4).
- **OBLIGATORIO**: mostrar quiénes están bajo umbral antes de calcular.
- Flag positivo: si una incorporación bajo umbral **está cumpliendo** el objetivo individual,
  destacarla como candidata a bonus excepcional ("se ha adaptado en tiempo récord").

## 5. Perfiles mixtos y overrides (por periodo, confirmable)

**Importante: los roles cambian entre trimestres.** No hardcodear overrides permanentes.
El rol de cada persona se detecta de Odoo (`job_id`/`department_id`) **para el periodo
analizado**, y los overrides se presentan como **confirmación por periodo**.

- El skill detecta el rol vigente en el periodo y SOLO incluye a quien pertenece a los
  3 equipos (Tokyo/Servicios/Mónaco) en ese periodo. Si alguien cambió a un equipo/rol
  fuera de esos 3 (p.ej. KAM), queda fuera de forma natural.
- **Aitor Díez Aniceto** (emp_id 45): ejemplo de perfil mixto **80/80** que aplica
  **solo en los periodos en que es consultor** (p.ej. Q1 2026). En Q2 2026 pasó a
  **Key Account Manager** → ya NO es consultor de esos equipos y no se le aplica 80/80.
  → Por eso NO se fija como regla permanente: se ofrece como override por defecto
  únicamente si en el periodo figura como consultor, y el usuario confirma o ajusta.
- El usuario puede añadir a cualquiera como perfil mixto (override `{individual, equipo}`)
  para ese periodo. Estos overrides son siempre un **ítem de confirmación explícito**
  en la validación previa (Paso 3).

## 6. Caso Ángel Angelini (doble simulación)

- Ángel (emp_id 60, user_id 425): PM del equipo de Servicios **y** consultor en Tokyo.
  En su ficha figura en **Servicios** (Consultor Senior).
- Apartado dedicado en el informe: **simular cómo quedarían Tokyo y Servicios** si se le
  contabiliza en cada uno, y **cuánto % del 20%** se llevaría él en cada caso.
- Las horas que se le aplican son **las mismas** en ambos equipos; lo que cambia es el
  objetivo: en un equipo objetivo de **consultor**, en el otro de **PM**.
- Mostrar el efecto sobre el tramo de equipo de cada pool (dónde se le imputa puede
  cambiar si Servicios/Tokyo alcanzan o no su objetivo de equipo).

## 7. [REPARTO] (solo análisis, NO suma)

- Detección: dominio `[["name","ilike","[reparto]"]]` (con corchetes).
- Las horas de reparto ya son **partes de hora dentro de la tarea** → ya están en
  `x_studio_bonus_unit_amount` del total. **NO se suman aparte.**
- `user_ids` de la tarea va vacío → los beneficiarios se obtienen agrupando
  `timesheet_ids` (account.analytic.line) por `employee_id`.
- Naming inconsistente en la práctica → **OBLIGATORIO mostrar la lista de tareas
  candidatas (con proyecto y horas) y ESPERAR la confirmación del usuario** antes de
  incluirlas en el análisis. No dar por buenas las detectadas sin validar.
- En el informe (sección 6): por empleado → horas repartidas, en qué proyectos, cuántas
  en cada uno, y **% que representan sobre su total** del periodo. Más **tipología** de
  los proyectos (migración/implantación/ampliación/análisis/soporte...) normalizada al
  vuelo desde `project.name` + `stage_id`.
