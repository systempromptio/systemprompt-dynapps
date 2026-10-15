# Etapas de Tickets y Periodos

## Etapas (helpdesk.stage)

| ID | Nombre | Tipo |
|----|--------|------|
| 1 | New | Abierto |
| 2 | In Progress | Abierto |
| 6 | En desarrollo | Abierto |
| 15 | Testeo Desarrollo | Abierto |
| 5 | Espera cliente | Abierto |
| 14 | Espera subida produccion | Abierto |
| 3 | Solved | Cerrado |
| 4 | Cancelled | Cerrado |

**IDs de etapas abiertas**: `[1, 2, 5, 6, 14, 15]`
**IDs de etapas cerradas**: `[3, 4]`

---

## Periodos en Curso

**IMPORTANTE**: Si el periodo solicitado incluye la fecha actual (periodo en curso):
- **Fecha fin = HOY**, no el final teorico del periodo
- Calcular dias laborables solo hasta HOY

**Ejemplos**:
- "Tickets cerrados esta semana" (hoy es jueves) -> Lunes a Jueves
- "Estadisticas de enero" (hoy es 15 enero) -> 1 enero a 15 enero
- "Estadisticas de enero" (hoy es 5 febrero) -> 1 enero a 31 enero (periodo cerrado)
