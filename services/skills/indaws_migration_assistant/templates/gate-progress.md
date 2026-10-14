# Template: Vista de Progreso de Gates

> Usar cuando el usuario pregunta: "¿Qué gates faltan?", "¿Qué gates tengo pendientes?",
> "Progreso de gates", "¿Cómo va la migración?" (sin datos de horas).
> Funcionalidad F9: consultar gates en Odoo y mostrar progreso visual.

---

## Migración {nombre_proyecto} — v{version_origen} → v{version_destino}

**Fase actual:** Fase {fase_num} — {fase_nombre}

### Estado de gates

| Gate | Nombre | Estado | Ejecuta | Aprueba | Fecha |
|------|--------|--------|---------|---------|-------|
| 0 | Alcance y Propuesta Comercial | {estado_gate_0} | PM | COO | {fecha_0} |
| 1 | Análisis Técnico | {estado_gate_1} | Dev | TL | {fecha_1} |
| 2 | Código Migrado | {estado_gate_2} | Dev | TL | {fecha_2} |
| 3 | Base de Datos Migrada | {estado_gate_3} | Dev | TL | {fecha_3} |
| 4 | Testing Funcional | {estado_gate_4} | BC + Dev | TL | {fecha_4} |
| 5 | Ensayo General (Rehearsal) | {estado_gate_5} | Dev | TL | {fecha_5} |
| 6 | Puesta en Producción | {estado_gate_6} | BC + Dev | TL + CTO Ops | {fecha_6} |
| 7 | Cierre del Proyecto | {estado_gate_7} | PM + TL + BC | COO | {fecha_7} |

**Leyenda:** Aprobado | Pendiente | (no creado)

```
Progreso: [{barra_progreso}] {porcentaje}% ({gates_aprobados}/8 gates)
```

### Próximo gate: Gate {proximo_gate_num} — {proximo_gate_nombre}

**Ejecuta:** {quien_ejecuta}
**Aprueba:** {quien_aprueba}

#### Entregables pendientes para Gate {proximo_gate_num}

| # | Entregable | Rol responsable |
|---|-----------|----------------|
| 1 | {entregable_1} | {rol} |
| 2 | {entregable_2} | {rol} |
| ... | | |

> Extraer entregables de `references/roles.md` para la fase del próximo gate.

### Tu situación ({rol_usuario})

{guia_personalizada_para_rol_en_fase_actual}

> Extraer de `references/roles.md` las acciones del rol del usuario en la fase actual.
> Incluir solo un resumen breve (2-3 acciones clave), no la guía completa.

---

## Si no hay gates creados

Si no se encuentran tareas [GATE] en el proyecto:

```
No se han encontrado tareas de gate en el proyecto {nombre_proyecto}.

¿Quieres que cree los 8 gates (Gate 0 a Gate 7)?
Esto creará una tarea por gate con la plantilla de validación y los entregables.
```

---

## Reglas de formato

- Datos de gates vía API (project.task search_read con filtro [GATE])
- Si un gate no existe como tarea, mostrarlo como "(no creado)"
- Si un gate existe pero no está aprobado, mostrarlo como "Pendiente"
- La fase actual se calcula según la lógica de "Detección de fase por gates" de SKILL.md
- Incluir siempre la guía personalizada según el rol del usuario
- Si el usuario no ha dicho su rol, omitir la sección "Tu situación" o preguntar
