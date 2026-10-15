# Template: Checklist de Fase

> Usar cuando el usuario pide: "Checklist fase X", "¿Qué hay que verificar?"
> Rellenar con datos de references/checklists.md

---

## Checklist — Fase {fase_num}: {fase_nombre}

**Proyecto:** {nombre_proyecto}
**Fecha:** {fecha}

### {nombre_checklist}

| # | Verificación | Responsable | Estado | Notas |
|---|---|---|---|---|
| 1 | {item_1} | {responsable} | ⬜ | |
| 2 | {item_2} | {responsable} | ⬜ | |
| ... | | | | |

**Leyenda:** ⬜ Pendiente | ✅ OK | ❌ KO | ⏭️ N/A

### Criterio de avance
{criterio_avance}

### Siguiente paso tras completar
{siguiente_paso}

---

## Reglas de formato

- Los ítems DEBEN provenir de references/checklists.md — NO inventar
- Si la fase tiene múltiples checklists (ej. Fase 4 tiene funcional + técnico, Fase 5 tiene infraestructura), mostrar TODOS
- Para Fase 5, preguntar si es Variante A (servidores propios) o Variante B (Odoo.sh) para mostrar el checklist correcto
- Estados posibles: ⬜ Pendiente / ✅ OK / ❌ KO / ⏭️ N/A (no aplica)
- Si el usuario ya tiene datos del proyecto, pre-rellenar lo que se pueda desde la API
