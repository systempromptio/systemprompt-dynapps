# Template: Resumen de Fase

> Usar cuando el usuario pregunta sobre una fase específica: "¿Qué se hace en Fase X?"
> Rellenar con datos de references/phases.md

---

## Fase {fase_num} — {fase_nombre}

### Objetivo
{objetivo_fase}

### Actividades principales

| # | Actividad | Responsable | Descripción |
|---|---|---|---|
| 1 | {actividad_1} | {responsable} | {descripcion} |
| 2 | {actividad_2} | {responsable} | {descripcion} |
| ... | | | |

### Roles involucrados

| Rol | Responsabilidad clave |
|---|---|
| PM | {responsabilidad_pm} |
| TL | {responsabilidad_tl} |
| BC | {responsabilidad_bc} |
| Dev | {responsabilidad_dev} |
| COO/CTO | {responsabilidad_coo_cto} |

### Gate {fase_num}: {nombre_gate}
- **Aprueba:** {quien_aprueba}
- **Ejecuta:** {quien_ejecuta}
- **Entregables obligatorios:**
  1. {entregable_1}
  2. {entregable_2}
  3. ...

### Dependencias con otras fases
- **Requiere completar:** Gate {fase_anterior} (Fase {fase_anterior})
- **Desbloquea:** Fase {fase_siguiente}

---

## Reglas de formato

- Toda la información DEBE provenir de references/phases.md
- NO inventar actividades ni entregables
- Si la fase tiene checklists asociados (Fase 4, 5), referenciarlos: "Ver checklist completo con: Checklist fase X"
- Incluir RACI si el usuario lo pide (de references/phases.md sección RACI)
