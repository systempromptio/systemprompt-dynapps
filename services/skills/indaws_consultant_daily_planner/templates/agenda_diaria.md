# Template: Agenda Diaria del Consultor

## Formato de la Agenda

```markdown
## AGENDA — {dia_semana} {fecha}

**Consultor**: {nombre_consultor}
**Proyectos activos**: {N} | **Horas planificadas este mes**: {total_horas}h

### Resumen
- {N_urgente} acciones urgentes | {N_importante} importantes | {N_planificado} planificadas
- Proyectos: {lista_proyectos_separados_por_coma}
{alertas_especiales}

---

### URGENTE (hoy)

**1. [{proyecto}] {titulo_descriptivo}** (~{tiempo}min)
   - {contexto_breve}
   - IDs: {request/tarea/ticket #X}
   → Ejecutar: `{skill}` {modo}, {parametros}
   {prerequisito_si_aplica}

**2. [{proyecto}] {titulo_descriptivo}** (~{tiempo}min)
   ...

---

### IMPORTANTE (si hay tiempo)

**N. [{proyecto}] {titulo_descriptivo}** (~{tiempo}min)
   - {contexto_breve}
   → Ejecutar: `{skill}` {modo}, {parametros}

---

### PLANIFICADO (esta semana)

**N. [{proyecto}] {titulo_descriptivo}** (~{tiempo}min)
   - {contexto_breve}
   → Ejecutar: `{skill}` {modo}, {parametros}

---

### Verificacion Knowledge

| Proyecto | Estructura | Estado |
|----------|-----------|--------|
| {proyecto_1} | {ok_o_detalle} | {emoji} |
| {proyecto_2} | {ok_o_detalle} | {emoji} |

---

### Resumen de Horas

| Proyecto | Planificadas | Consumidas | % | Estado |
|----------|-------------|-----------|---|--------|
| {proyecto_1} | {plan}h | {real}h | {pct}% | {ok/alerta} |
| {proyecto_2} | {plan}h | {real}h | {pct}% | {ok/alerta} |
| **Total** | **{total_plan}h** | **{total_real}h** | **{total_pct}%** | |
```

## Reglas del Template

### Alertas Especiales (debajo del resumen)
Incluir si aplica:
- `> Es viernes — ejecutar project-weekly-update en todos los proyectos`
- `> Go-Live de {proyecto} en {N} dias`
- `> Health score CRITICO en {proyecto} ({score}/100)`
- `> {N} tickets abiertos sin respuesta`

### Emojis de Estado Knowledge
- OK: sin problemas
- Falta "{articulo}": indicar que falta
- Verificar: necesita revision manual

### Seccion de Horas
- Solo mostrar si hay datos de timesheets disponibles
- Alertar si consumo > 80% de planificadas con semanas restantes
- Alertar si consumo < 30% y ya paso el 50% del mes

### Si No Hay Acciones en un Nivel
- Omitir la seccion entera (no mostrar "No hay acciones urgentes")
- Si no hay nada urgente ni importante, el resumen refleja solo planificadas
