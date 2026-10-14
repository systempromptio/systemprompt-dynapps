# Frecuencia Inteligente - Reglas Anti-Molestia

## Principio

El Mentor debe ser **útil, no molesto**. Estas reglas evitan la fatiga de alertas y garantizan que cada notificación aporte valor real al usuario.

> Si el usuario siente que el Mentor le interrumpe demasiado, hemos fallado.

---

## Reglas de Supresión

### 1. Deduplicación diaria

Si la **misma alerta** ya se emitió hoy → **suprimir**, EXCEPTO si ha empeorado.

**Definición de "misma alerta":**
- Mismo `project_id` o `opportunity_id`
- Mismo nivel de urgencia (CRITICAL, WARNING, MONITOR)
- Mismo dominio (Proyecto, CRM, Equipo)

**Cuándo se re-emite:**
- Si el nivel ha empeorado (ej: WARNING → CRITICAL)
- Si han aparecido datos nuevos relevantes (ej: nueva tarea vencida en el mismo proyecto)

**Ejemplo:**
```
09:00 — 🟠 [AVISO] Proyecto "Portal Web" score 65 → se emite
12:00 — 🟠 [AVISO] Proyecto "Portal Web" score 63 → se SUPRIME (mismo nivel, delta < 5)
15:00 — 🔴 [CRÍTICO] Proyecto "Portal Web" score 38 → se EMITE (empeoró a CRITICAL)
```

---

### 2. Umbral de delta

Si el cambio de score es **menor de 5 puntos** → **no alertar** (se considera fluctuación normal).

| Delta | Acción |
|-------|--------|
| < 5 puntos | Suprimir. No alertar. |
| >= 5 puntos | Alertar según nivel. |
| Cruza umbral (80→79, 70→69, 40→39) | **Alertar SIEMPRE**, independientemente del delta. |

**Umbrales críticos de cruce:**
- **80 puntos**: frontera HEALTHY/MONITOR
- **70 puntos**: frontera MONITOR/WARNING
- **40 puntos**: frontera WARNING/CRITICAL

**Ejemplo:**
```
Score pasa de 82 a 79 (delta = -3, pero cruza umbral 80) → se EMITE
Score pasa de 74 a 71 (delta = -3, no cruza umbral) → se SUPRIME
Score pasa de 71 a 65 (delta = -6, cruza umbral 70) → se EMITE
```

---

### 3. Cooldown post-CRITICAL

Después de emitir una alerta **CRITICAL** → **esperar 2 horas** antes de emitir otra alerta del mismo ítem.

**Lógica:**
```
última_alerta_critical[item_id] = timestamp
si (ahora - última_alerta_critical[item_id]) < 2h → SUPRIMIR
```

**Excepción:** Si el score baja **aún más** durante el cooldown → alertar inmediatamente.

**Ejemplo:**
```
09:00 — 🔴 [CRÍTICO] Proyecto "Migración" score 35 → se emite
10:00 — 🔴 [CRÍTICO] Proyecto "Migración" score 34 → se SUPRIME (cooldown activo, delta < 5)
10:30 — 🔴 [CRÍTICO] Proyecto "Migración" score 28 → se EMITE (caída adicional > 5 puntos)
```

---

### 4. Límite de elogios

- **Máximo 1 elogio por día**, y solo en el **briefing matutino** (09:00)
- Un elogio = mensaje positivo sobre mejora de score o logro

**Qué cuenta como elogio:**
- "Tu proyecto X subió a 85, ¡buen trabajo!"
- "Pipeline CRM saludable, win rate por encima del objetivo"
- "Todas las horas imputadas al día esta semana"

**Qué NO es un elogio (y se puede repetir):**
- Confirmación de estado HEALTHY sin mejora (dato neutro, no elogio)
- "Sin novedades" (informativo, no elogio)

**No repetir elogios** en revisiones de mediodía, tarde o fin de día.

---

### 5. Modo minimal

Si el usuario configura `minimal_mode = true` en `preferences.json`:

| Aspecto | Modo normal | Modo minimal |
|---------|-------------|--------------|
| Alertas CRITICAL | ✅ Se muestran | ✅ Se muestran |
| Alertas WARNING | ✅ Se muestran | ✅ Se muestran |
| Alertas MONITOR | ✅ Se muestran | ❌ Se suprimen |
| Estado HEALTHY | ✅ Se muestran | ❌ Se suprimen |
| Tablas detalladas | ✅ Completas | ❌ Solo resumen en una línea |
| Recomendaciones de skills | ✅ Hasta 3 | ✅ Solo 1 (la más urgente) |
| Elogios | ✅ 1 por día | ❌ Suprimidos |

---

### 6. Silencio inteligente

Cuando **no hay cambios** desde la última revisión, el comportamiento varía según la hora del día:

| Hora del día | Sin cambios | Comportamiento |
|--------------|-------------|----------------|
| Mañana (09:00) | Sin cambios desde ayer | Emitir briefing completo con scores actuales. Es el inicio del día, el usuario necesita contexto. |
| Mediodía (~12:00) | Sin cambios desde mañana | **"Sin novedades desde las 09:00. ¡Sigue así! 💪"** |
| Tarde (~15:00) | Sin cambios desde mediodía | **"Sin novedades desde las 12:00. ¡Sigue así! 💪"** |
| Fin de día (~18:00) | Sin cambios intradía | Emitir resumen del día (aunque sea "día tranquilo"). El usuario necesita cierre. |

> El silencio inteligente es la herramienta más potente contra la fatiga de alertas. Si no hay nada que decir, **no digas nada** (excepto mañana y fin de día).

---

## Parámetros configurables (preferences.json)

| Parámetro | Tipo | Default | Descripción |
|-----------|------|---------|-------------|
| `silence_threshold` | `int` | `5` | Puntos mínimos de delta para generar una alerta. Valores más altos = menos alertas. |
| `critical_cooldown_hours` | `int` | `2` | Horas de espera después de una alerta CRITICAL antes de volver a alertar del mismo ítem. |
| `minimal_mode` | `bool` | `false` | Si `true`, solo muestra alertas CRITICAL y WARNING. Resúmenes más cortos. |
| `excluded_projects` | `list[int]` | `[]` | IDs de proyectos a ignorar completamente (no aparecen en ningún informe). |
| `excluded_stages` | `list[str]` | `[]` | Etapas CRM a ignorar (ej: `["Nuevo", "Perdido"]`). |
| `max_recommendations` | `int` | `3` | Número máximo de recomendaciones de skills por revisión. |
| `praise_enabled` | `bool` | `true` | Si `false`, suprime todos los elogios. |

**Ejemplo de preferences.json:**
```json
{
  "silence_threshold": 5,
  "critical_cooldown_hours": 2,
  "minimal_mode": false,
  "excluded_projects": [142, 287],
  "excluded_stages": ["Nuevo"],
  "max_recommendations": 3,
  "praise_enabled": true
}
```
