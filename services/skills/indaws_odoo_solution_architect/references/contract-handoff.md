# Contrato de Handoff (Step 5)

El último paso: dejar **inequívoco** qué recibe el dev, qué es intocable y qué decide. Un handoff
ambiguo es la causa nº1 de que el dev se desvíe del diseño o de que pregunte a mitad.

---

## 1. Contrato fijo vs libertad del dev (explícito, en la ficha)

Dos listas, sin zona gris:

**Contrato (los planos) — NO se cambia sin volver al arquitecto:**
- Firmas de método (nombre, parámetros, tipo de retorno).
- Nombres/tipos/`store` de campos del contrato.
- Fronteras y dependencias del módulo.
- Puntos de integración (eventos, hooks, señales).
- Criterios de aceptación (QUÉ verificar) — el dev los cubre con tests, pero no cambia el criterio.
- El pseudocódigo describe QUÉ debe hacer cada cuerpo (el dev no cambia el qué, sí el cómo).

**Libertad del dev (el cemento) — su criterio:**
- El código real de los cuerpos a partir del pseudocódigo.
- Nombres de variables locales y helpers privados.
- Orden de operaciones dentro del contrato.
- Optimizaciones que no cambien la interfaz.

> Si el dev cree que el contrato está mal → **lo discute, no lo cambia unilateralmente**. Cambiar el
> contrato es volver a Step 2 (re-diseño), no un parche local.

## 2. Puntos de integración (el walking skeleton)

Marcar dónde este trabajo se engancha con lo existente:
- Qué modelo/método core se extiende y **por dónde** (el `super()` que se respeta).
- Qué eventos dispara / consume (mail, actividades, automatizaciones).
- Qué otros módulos/tareas dependen de esta interfaz (para que no rompan en paralelo).
- **La primera línea end-to-end que debe andar** — el dev empieza por ahí, no por las hojas.

## 3. Definition of Ready (gate antes de delegar)

No se entrega la tarea si falta algo:

- [ ] Contexto de negocio (POR QUÉ) y ≥2 escenarios concretos.
- [ ] Versión Odoo objetivo confirmada.
- [ ] Mecanismo central trazado a `file:line` (o probe con rollback).
- [ ] Esqueleto con firmas + docstrings de contrato + pseudocódigo de los cuerpos.
- [ ] Núcleo de riesgo en pseudocódigo + cita a fuente `file:line` verificada.
- [ ] Criterios de aceptación definidos (QUÉ verificar, comportamiento).
- [ ] Contrato fijo vs libertad del dev, explícito.
- [ ] <3 preguntas abiertas (si más → no está listo).

## 4. Definition of Done (qué se le exige al dev)

- [ ] **Tests unitarios** que cubren todos los criterios de aceptación, en verde.
- [ ] **Flujo(s) end-to-end** que ejercitan el caso de negocio completo, en verde.
- [ ] Contrato respetado (firmas/campos intactos).
- [ ] Checklist de impacto Odoo cubierto (datos existentes, multi-company, seguridad, upgrade, perf).
- [ ] Sin deps nuevas no acordadas en `__manifest__`.

> Los tests (unit + e2e) los escribe y ejecuta el dev — es el "cómo verificar". Nosotros exigimos la
> cobertura y definimos el "qué verificar" (criterios). Sin tests no hay entrega.

## 5. Empaquetado final → `dev-task-writer`

El diseño + esqueleto no se entregan crudos: se pasan a `dev-task-writer`, que los envuelve en la
tarea formal (7 secciones, checklist de impacto, estimación con banda). El esqueleto va como **rama
base / entregable adjunto**, y el contrato fijo/libre se refleja en las secciones "Guía de
Implementación" (libertad) y "Restricciones" (contrato) de la tarea.

Reconciliación con la regla de dev-task-writer "no dar código": lo que se entrega son **planos**
(esqueleto + pseudocódigo + tracer bullet en pseudocódigo con cita a fuente), **no cemento** (código
Python ejecutable). El dev sigue siendo quien pone el cemento y resuelve el CÓMO.
