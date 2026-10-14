# Handoff de Implementación — {título}

> Entregable del arquitecto al dev. Va acompañado del **esqueleto** (rama base) y del **diseño**
> (`solution-design.md`). Este documento se pasa a `dev-task-writer` para el empaquetado formal.

**Módulo**: `{técnico}` | **Odoo**: {X.0} | **Rama base con esqueleto**: `feature/{nombre}-skeleton`
**Asignado**: {dev} | **Nivel**: {Junior/Mid/Senior} | **Estimación**: {banda: probable Yh · peor Zh}

## Qué recibes ya hecho (los planos)

- Diseño de solución + ADR (`solution-design.md`).
- Esqueleto de módulo: modelos/campos/firmas con **docstrings de contrato + pseudocódigo** (`# PSEUDO`).
- **Núcleo de riesgo en pseudocódigo + cita a fuente** (`# TRACER BULLET`) — el mecanismo está
  verificado contra la fuente; tú pones el cemento.
- **Criterios de aceptación** (QUÉ verificar) — los tests que los cubren los escribes tú.

> Recibes los planos y el detalle, no el cemento: el código real de los cuerpos lo escribes tú a
> partir del pseudocódigo, respetando las firmas y los contratos.

## Contrato — NO se cambia sin volver al arquitecto

- {firmas de método fijas}
- {campos: nombre/tipo/store fijos}
- {puntos de integración}
- {criterios de aceptación}

## Tu libertad (DECIDE-DEV) — el cemento

- {el código real de los cuerpos a partir del pseudocódigo}
- {nombres locales, helpers privados}
- {optimizaciones que no toquen la interfaz}

## Puntos de integración

- **Empieza por**: {la primera línea end-to-end del walking skeleton}
- **Extiende**: `{modelo/método core}` respetando `super()`
- **Dispara/consume**: {eventos, mail, actividades}

## Definition of Done (se te exige)

- [ ] **Tests unitarios** que cubren los criterios de aceptación, en verde.
- [ ] **Flujo(s) end-to-end** del caso de negocio, en verde.
- [ ] Contrato respetado (firmas/campos intactos).
- [ ] Checklist de impacto Odoo cubierto.
- [ ] Sin deps nuevas no acordadas.

## Criterios de aceptación (QUÉ verificar — tú escribes los tests)

- [ ] {Cuando X con valores concretos, entonces Y}
- [ ] {Cuando edge case Z, entonces W}

## Preguntas abiertas

- {si >3 → la tarea no está lista, no se delega}

## Ayuda

- Timer: pide ayuda si llevas {Junior 1.5h / Mid 4h / Senior 1 día} atascado.
- Contacto: {arquitecto} por {canal}. Si crees que el contrato está mal → discútelo, no lo cambies.

## Revisión (protocolo — obligatorio)

1. Cuando termines, abre el **PR** y **notifícanos ahí** (mención/asignación de review a {arquitecto}).
2. Revisamos contra el contrato y los criterios de aceptación (tests unit + e2e en verde).
3. Resultado: **merge** (aprobado) o **correcciones** (te devolvemos el review con los cambios).
4. La tarea NO se cierra sin esa notificación en el PR y sin nuestra aprobación.
