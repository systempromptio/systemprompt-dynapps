# Template: Guía por Perfil

> Usar cuando el usuario pregunta: "¿Qué tengo que hacer?", "Soy [rol], ¿qué hago?"
> Rellenar con datos de `references/roles.md` (acciones numeradas + entregables).

---

## Eres {rol} en Fase {fase_num} — {fase_nombre}

### Tus acciones en esta fase

| # | Acción | Entregable | Estado |
|---|--------|-----------|--------|
| 1 | {accion_1} | {entregable_1} | ⬜ |
| 2 | {accion_2} | {entregable_2} | ⬜ |
| ... | | | |

> Extraer acciones numeradas de `references/roles.md`, sección del rol en la fase.
> Cada acción tiene su entregable concreto — incluirlo siempre.

### Comunicación requerida

| De → A | Frecuencia | Formato |
|--------|-----------|---------|
| {origen} → {destino} | {frecuencia} | {formato_o_template} |

> Extraer de `references/communication.md` las filas relevantes para este rol.

### Gate de esta fase — Gate {gate_num}: {nombre_gate}
- **Ejecuta:** {quien_ejecuta}
- **Aprueba:** {quien_aprueba}
- **Tu entregable para este gate:** {entregable_especifico_del_rol}

> Si el rol es quien ejecuta el gate, indicar que debe rellenar la plantilla de
> `references/gate-templates.md`. Si es quien aprueba, indicar qué debe verificar.

### Referencia rápida
- Para ver la plantilla del gate: "Gate {gate_num}"
- Para ver el checklist de esta fase: "Checklist fase {fase_num}"
- Para ver el resumen de la fase: "¿En qué fase estoy?"

---

## Reglas de formato

- Extraer acciones de `references/roles.md` — NO resumir, usar las acciones tal cual
- Incluir SIEMPRE el entregable concreto de cada acción
- Incluir comunicación de `references/communication.md` filtrada por rol
- Incluir el gate de la fase con el rol del usuario destacado
- Si el usuario no ha indicado fase, preguntar antes de responder
