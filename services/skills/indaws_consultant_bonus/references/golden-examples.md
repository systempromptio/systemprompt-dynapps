# Golden Examples — consultant-bonus

**Ancla de regresión: antes de modificar las reglas, confirma que estos ejemplos siguen dando el mismo resultado. Si cambian a propósito, actualiza el ejemplo.**

Casos canónicos con datos sintéticos (sin empleados reales). Los valores se derivan de
`reglas-bonus.md` §1,§3,§4,§5,§7. Trimestre cerrado, meses=3. El cálculo real lo hace
`scripts/calculate_bonus.py`; aquí se fija el resultado esperado para detectar drift.

---

## Ejemplo 1 — Consultor con objetivo individual cumplido al ~90% → tramo 0.5

### INPUT (1 consultor, trimestre cerrado, meses=3, `frac_activa=1.0`)
- Rol: **consultor** (topes: individual 0.14, equipo 0.06).
- Objetivo individual = 113 h/mes × 3 = **339 h**.
- Horas computables del empleado = **305 h** (≈ 90% del objetivo).
- Su equipo, en bloque equipo, cumple `pct_equipo ≥ 1.00` → tramo_equipo = 1.0.

### OUTPUT ESPERADO
- `pct_individual = 305 / 339 = 0.900` → cae en **0.80 ≤ pct < 1.00** ⇒ **tramo_individual = 0.5**.
- `var_individual = 0.5 × 0.14 = 0.07` (7% del sueldo).
- `var_equipo = 1.0 × 0.06 = 0.06` (6% del sueldo).
- `var_total = 0.07 + 0.06 = 0.13` (≤ 0.20 ✓).
- `var_en_juego_pct = 0.13 / 0.20 × 100 = 65.0`.

**Invariantes:**
- Tramo individual ∈ {0.0, 0.5, 1.0} — el 90% cae en el medio (0.5), nunca un valor intermedio.
- `var_total ≤ 0.20` siempre; consultor tope individual 0.14 / equipo 0.06.
- Objetivo individual = 113 (no 105 ni 145); 145 es jornada, NO objetivo.

---

## Ejemplo 2 — Incorporación con prorrateo por días (medio trimestre)

### INPUT (1 consultor incorporado a mitad de trimestre)
- Rol: **consultor**. Trimestre cerrado, días_totales = 90, días activos = 45.
- `frac_activa = 45/90 = 0.5` ⇒ `meses_activos = 3 × 0.5 = 1.5`.
- Perfil **senior** (`job_id` con experiencia): umbral de cobro = 1 mes. Como sus meses
  activos (1.5) superan el umbral ⇒ `cuenta_bonus = true`, `cobra = true`.
- Horas computables = 180 h.

### OUTPUT ESPERADO
- Objetivo individual prorrateado = 113 × meses_activos = 113 × 1.5 = **169.5 h**.
- `pct_individual = 180 / 169.5 = 1.062` → **pct ≥ 1.00** ⇒ **tramo_individual = 1.0**.
- `var_individual = 1.0 × 0.14 = 0.14`.
- Su objetivo entra en el **denominador del equipo prorrateado**: `objetivo_rol × meses × frac`
  = 105 × 3 × 0.5 = **157.5 h** (no los 315 h de un miembro a tiempo completo).
- Variante: si fuese **junior** con < 6 meses activos ⇒ `cuenta_bonus = false`:
  sus horas SÍ suman al numerador del equipo, su objetivo NO entra al denominador, `var_total = 0`.

**Invariantes:**
- El prorrateo se aplica por DÍAS vía `frac_activa` (0..1), tanto al objetivo individual como al aporte de equipo.
- Senior bajo umbral = 1 mes; junior = 6 meses; bajo umbral ⇒ `cuenta_bonus=false` y no cobra.
- Becario/incorporación no establecida: horas al numerador, objetivo fuera del denominador (no penaliza al pool).
