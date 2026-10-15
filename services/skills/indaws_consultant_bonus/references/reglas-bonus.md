# Reglas de cálculo del bonus trimestral — consultant-bonus

Toda la lógica matemática vive en `scripts/calculate_bonus.py`. El modelo (LLM)
**nunca calcula a mano**: trae los datos por MCP, los estructura en el JSON de
entrada y ejecuta el script. Este archivo define las reglas y el contrato de E/S.

## 1. Variable y objetivos (valores por defecto — el skill PREGUNTA si cambiarlos)

El variable es el **20% del sueldo**, repartido en dos bloques. En el informe el "%
variable" se muestra como **% del variable en juego (0-100%)** (decisión del usuario);
el motor también devuelve la fracción de sueldo (máx 0.20). Sin € salvo que se aporten sueldos.

| Rol | Obj. INDIVIDUAL /mes | Obj. EQUIPO /mes | Split individual | Split equipo |
|-----|----------------------|------------------|------------------|--------------|
| Consultor   | 113 | 105 | 70% → tope 14% sueldo | 30% → tope 6% sueldo |
| PM          | 80  | 80  | 40% → tope 8% sueldo  | 60% → tope 12% sueldo |
| Desarrollo  | —   | 85  | — (no cobra)          | — (suma al objetivo de equipo) |

> Objetivos **mensuales** (×nº meses del periodo; trimestre = ×3). Individual y equipo
> usan horas DISTINTAS (consultor 113 individual / 105 equipo).
> ⚠️ Las **145h/mes** del PDF son la *jornada* (capacidad) para expresar "facturabilidad",
> **NO** el objetivo. El objetivo de equipo son las **105/85/80 h facturables por rol**
> (PDF §Bloque de Equipo: "el promedio del equipo facture al menos 85 dev / 80 pm / 105 consultor").
> El "% variable en juego" = `var_total / 0.20 × 100`.

## 2. Horas computables

`computables(empleado) = Σ x_studio_bonus_unit_amount (ya con ×0.7)  +  demos`

- **Demos**: `+4h ÷ nº participantes` por cada **OPORTUNIDAD** con al menos una actividad Demo
  cerrada (`date_done`) en el periodo donde participa el empleado. Fuente: `mail.activity`, no
  `crm.lead` — los campos de Studio fueron borrados. **Agrupar por `res_id`: una oportunidad
  cuenta UNA demo aunque tenga varias actividades** (Q2 2026: 96 actividades → 79 oportunidades).
  Los participantes son los `user_id` distintos de las actividades de esa oportunidad. Planas
  (sin 0.7). Se suman al total del empleado, en su equipo (`hr.employee.x_studio_team_id`).
- **[REPARTO]**: NO se suma aparte — esas horas ya son partes dentro de la tarea y
  ya están en `x_studio_bonus_unit_amount`. Solo se ANALIZAN (sección 6 del informe).

## 3. Bloque INDIVIDUAL

```
obj_ind   = obj_individual_rol × meses
pct_ind   = computables / obj_ind
tramo_ind = escala(pct_ind)        # ver §5
%_variable_individual = tramo_ind × tope_individual_rol   # 14% consultor / 8% PM
```
Desarrollo no tiene bloque individual.

## 4. Bloque EQUIPO (% COMPLETADO DEL OBJETIVO POR ROL)

El objetivo de equipo es el **% completado del objetivo de horas por rol** (105 consultor /
85 dev / 80 PM). NO es facturabilidad sobre 145h (eso es solo la jornada/capacidad del PDF):

```
# NUMERADOR: horas computables de TODOS los miembros del equipo
real_equipo = Σ computables(m)                                    para TODO miembro m
# DENOMINADOR: Σ objetivo por rol × meses × frac, SOLO de ESTABLECIDOS (cuenta_bonus = true)
obj_equipo  = Σ (objetivo_rol(m) × meses × frac_activa(m))        para m con cuenta_bonus = true
pct_equipo  = real_equipo / obj_equipo                            # % completado del objetivo
tramo_equipo = escala_equipo(pct_equipo)                          # ver §5 (= escala individual)
%_variable_equipo(empleado) = tramo_equipo × tope_equipo_rol(empleado)  # 6% consultor / 12% PM
```
**Quién cuenta en el denominador (decisión del usuario):** SOLO los **establecidos**
(`cuenta_bonus = true`: consultores, PM y devs). Becarios/incorporaciones bajo umbral
(`cuenta_bonus = false`): **sus horas SÍ suman al numerador**, pero su **objetivo NO entra
en el denominador** (no penalizan al pool) y **no cobran**. Prorrateo por días vía `frac_activa`.

> Las **145h** del PDF son solo la jornada para "facturabilidad"; NO entran en el cálculo
> del bonus. Los objetivos por rol y los topes son parametrizables en `config` (ver §9).

## 5. Escala de tramos (sin huecos)

Misma escala para individual y equipo (el equipo se mide como **% del objetivo por rol**):

```
  pct >= 1.00            -> 1.0   (100% del tramo)
  0.80 <= pct < 1.00     -> 0.5   (50%)
  pct < 0.80             -> 0.0
```

## 6. Variable total y "% en juego"

```
%_variable_total(empleado) = %_variable_individual + %_variable_equipo   # máx 0.20 del sueldo
%_variable_en_juego        = var_total / 0.20 × 100                        # 0-100% (lo que se muestra)
```
PM y consultor cobran; desarrollo y `cuenta_bonus=false` → var_total = 0 (pero se calculan
sus % para el flag "rinde bien").

## 7. Prorrateo POR DÍAS (bajas, incorporaciones, cambios de equipo)

Cada empleado lleva `frac_activa` (0..1) = días activos en el periodo / días del periodo.
`meses_activos = meses × frac_activa` (fraccional). Prorratea el objetivo individual y
el aporte al objetivo de equipo (objetivo_rol × meses_activos).

- **Baja** (res.users.write_date): `frac_activa` = (días desde inicio del periodo hasta la baja) / días del periodo.
- **Cambio de equipo**: una entrada por (empleado × equipo) con su `frac_activa` y las
  horas hechas en cada equipo (no hay fecha en Odoo → se pregunta).
- **Incorporación** (create_date): umbral = 1 mes si `job_id.name` tiene experiencia
  (senior/medior), 6 meses si junior. Bajo el umbral: `cuenta_bonus=false`. (Su `frac_activa`
  refleja los días trabajados.)

## 8. Alcance temporal (modo del periodo)

- `trimestre_cerrado`: el trimestre indicado, tal cual.
- `trimestre_curso`: datos hasta hoy + **previsión** por extrapolación lineal según
  días laborables transcurridos del trimestre (`previsto = real × dias_totales / dias_transcurridos`).
- `anual`: desglose por los 4 trimestres + media de cumplimiento de los trimestres
  (acumulativo: un buen Q compensa otro malo, para regularización y alerta de vacaciones).

---

## 9. CONTRATO DE E/S de `calculate_bonus.py`

El script lee **JSON por stdin** y escribe **JSON por stdout**. Es PURO: no se conecta
a Odoo. Formato de horas en la salida legible: **HH:MM** (nunca decimales), pero los
campos numéricos crudos van en float para poder reusarlos.

### Entrada (stdin)
```json
{
  "periodo": {"modo": "trimestre_cerrado|trimestre_curso|anual",
              "year": 2026, "quarter": 1, "meses": 3,
              "dias_transcurridos": 40, "dias_totales": 64},
  "config": {
    "objetivos_individual": {"consultor": 113, "pm": 80, "desarrollo": null},
    "objetivos_equipo":     {"consultor": 105, "pm": 80, "desarrollo": 85},
    "topes":    {"consultor": {"individual": 0.14, "equipo": 0.06},
                  "pm": {"individual": 0.08, "equipo": 0.12}},
    "variable_total": 0.20,
    "umbrales_individual": [[1.00, 1.0], [0.80, 0.5]],   // opcional (default)
    "umbrales_equipo":     [[1.00, 1.0], [0.80, 0.5]]    // opcional (default; = individual)
  },
  "empleados": [
    {"emp_id": 199, "user_id": 1133, "nombre": "María Erades",
     "equipo": "monaco", "rol": "consultor",
     "frac_activa": 1.0,                      // días_activos/días_periodo (default 1.0)
     "cuenta_bonus": true,
     "override": null,                        // p.ej. {"individual":80} (perfil mixto por periodo)
     "horas_bonus": 304.0,                    // Σ x_studio_bonus_unit_amount del periodo (su equipo)
     "demos_h": 4.0,                          // +4/N ya repartidas
     "reparto_h": 12.0,                       // SOLO informativo (ya incluido en horas_bonus)
     "proyectos_penalizados": 2               // SOLO informativo, para alertas
    }
  ]
}
```
> En vez de `frac_activa` se pueden pasar `dias_activos` + `dias_periodo`. Para cambios de
> equipo: una entrada por (empleado × equipo). En modo `anual`: clave `trimestres` (lista),
> cada elemento = un periodo con su `empleados` (ver docstring del script).

### Salida (stdout)
```json
{
  "periodo": {...echo...},
  "empleados": [
    {"emp_id": 199, "nombre": "María Erades", "equipo": "monaco", "rol": "consultor",
     "cuenta_bonus": true, "cobra": true,
     "computables": 308.0, "computables_hhmm": "308:00",
     "obj_individual": 339.0, "pct_individual": 0.908, "tramo_individual": 0.5,
     "pct_equipo": 0.63, "tramo_equipo": 1.0,
     "var_individual": 0.07, "var_equipo": 0.06, "var_total": 0.13,
     "var_en_juego_pct": 65.0}
  ],
  "equipos": {
    "monaco": {"obj_equipo": 3150.0, "real_equipo": 3290.0,
               "pct_equipo": 1.044, "pct_equipo_display": 104.4, "tramo_equipo": 1.0,
               "n_miembros": 12, "personas_equiv": 11.5, "media_persona": 274.2}
  }
}
```
- `pct_equipo` es el **% completado del objetivo por rol** (real / Σ objetivos 105·85·80).
- `var_en_juego_pct` (0-100) es lo que se muestra en la columna "% variable" del informe.
- Si `modo == trimestre_curso`: se añade un bloque `prevision`. `cuenta_bonus=false` →
  `cobra=false` y `var_*`=0, pero los `pct_*` se calculan igual (flag "rinde bien").

### Entrada del modo ANUAL (formato fijado)
En modo anual la entrada NO es la lista plana de `empleados`, sino una lista de trimestres,
cada uno con el MISMO formato que un trimestre suelto (su propia lista de `empleados`):
```json
{"periodo": {"modo": "anual", "year": 2026},
 "config": {...},
 "trimestres": [
   {"quarter": 1, "meses": 3, "empleados": [...]},
   {"quarter": 2, "meses": 3, "empleados": [...]},
   ...
 ]}
```
Salida: `empleados[].trimestres[]` (detalle por Q) + `pct_*_anual` / `var_*` promediados,
y `trimestres_detalle[]` con el cálculo completo de cada Q.

### Cambios de equipo en el JSON
El script NO parte horas entre equipos por sí mismo: el modelo debe emitir **una entrada
por cada (empleado, equipo)** con su `meses_activos` y sus `horas_bonus` ya prorrateados.
Ej.: 1 mes en Tokyo + 2 en Mónaco → dos entradas del mismo empleado (Tokyo meses_activos=1,
Mónaco meses_activos=2) con las horas que hizo en cada equipo.

### Nota sobre `splits` y `topes`
Los `topes` ya incorporan `split × 20%` (consultor 0.14/0.06, PM 0.08/0.12). `splits` se
mantiene en el contrato solo por trazabilidad; el script usa `topes` directamente.
