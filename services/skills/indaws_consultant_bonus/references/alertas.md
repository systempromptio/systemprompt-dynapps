# Alertas por empleado — consultant-bonus

Las alertas se generan a partir del cálculo de partes/objetivos (no son intuiciones).
En el informe salen como **chips de 1 línea** con color semáforo. Solo aparecen si aplican.

## Tipos de alerta

### 🔴 Penalización 0.7 recurrente
- El empleado no llega al objetivo porque sus proyectos están penalizados (imputa fuera
  de plazo → el campo bonus le resta el 0.7).
- Detección (M1): la penalización por empleado = `unit_amount − x_studio_bonus_unit_amount`,
  AMBOS ya disponibles en el read_group principal por `employee_id`. Si esa brecha es alta
  y `pct_individual < 1.0`, alerta. **NUNCA** hacer un read_group por proyecto/tarea para
  esto (explota a cientos de miles de filas).
- `proyectos_penalizados` en el JSON es solo un contador informativo opcional.
- Ejemplo de texto: *"María Erades: −X h por imputar fuera de plazo (0.7)."*

### 🟠 PM sobre-ejecutando
- Un PM mete muchas más horas facturables que su objetivo (debería gestionar, no ejecutar).
- Detección: `rol == pm` y `computables > obj_individual × 1.3` (umbral orientativo).
- Ejemplo: *"Anna: 120h / 80h objetivo — centrarse en gestión de equipo."*

### 🟡 Riesgo de vacaciones / objetivo anual justo
- El empleado va **justo en el objetivo ANUAL** y aún tiene muchos días de vacaciones por
  coger → riesgo de no llegar al pedir vacaciones en lo que queda de año.
- Datos (modo anual): `hr.leave` validadas del año + días restantes
  (`virtual_remaining_leaves`) + `pct_anual` cercano a 1.0 o por debajo.
- Sub-casos:
  - **No ha cogido vacaciones** (sin hr.leave `validate`) + va justo → riesgo alto.
  - **Planificadas sin aprobar** (hr.leave `confirm`) → riesgo medio.
  - **Sin planificar nada** (sin registros futuros) → más imprevisible, avisar.
- Ejemplo: *"Juan: anual al 98% y 18 días de vacaciones sin planificar — riesgo."*

### 🟢 Rinde muy bien / candidato a bonus
- Cualquier empleado con rendimiento destacado que conviene ver para **decidir el bonus
  humanamente**. En especial:
  - Incorporación bajo umbral (`cuenta_bonus=false`) que **cumple** el objetivo individual
    → *"Laura (nueva): cumpliendo objetivo — candidata a bonus excepcional."*
  - Cualquiera muy por encima del objetivo de forma sana.

## Reglas de presentación

- Una línea por alerta, sin párrafos. Nombre + dato concreto + acción/lectura breve.
- Ordenar por gravedad: 🔴 → 🟠 → 🟡 → 🟢.
- Si no hay alertas de un tipo, no mostrar nada de ese tipo (sin "no hay alertas").
- Las alertas de vacaciones/anual solo tienen sentido en modo `anual` (o si el usuario
  aporta el contexto anual). En modo trimestre, omitir las de riesgo anual.
