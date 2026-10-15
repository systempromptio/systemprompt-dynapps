# Metodología de Arquitectura para Delegar (destilada)

Base teórica del skill. No es relleno académico: cada concepto se traduce en una regla operativa.

---

## 1. Spec-Driven Development (SDD) — el contrato es la fuente de verdad

La especificación (diseño + esqueleto) es la fuente de verdad; el código del dev es artefacto que
la satisface. GitHub Spec Kit (2025) lo formaliza en **Specify → Plan → Tasks → Implement**, que es
el workflow de este skill (Specify → Design → Skeleton → Handoff).

- **Regla**: cuando el trabajo cambia de manos (arquitecto → dev, sesión → sesión), **el spec es el
  artefacto de handoff**. Si algo no está en el esqueleto o su docstring de contrato, el dev lo
  decide — y hay que ser consciente de qué le estamos dejando decidir.
- Fuente: Thoughtworks / Microsoft / InfoQ (SDD 2025), GitHub Spec Kit.

## 2. Contract-First — "válido por definición"

Acordado el contrato (interfaces, firmas, tipos, invariantes), **cualquier implementación que lo
respete y cumpla los criterios de aceptación es válida**. El dev no puede desviarse del diseño sin romper el
contrato, y no necesita permiso para elegir el CÓMO dentro de él.

- **Regla**: toda firma lleva **docstring de contrato**: entradas, salida, invariantes, efectos
  laterales, qué excepciones lanza. Un stub sin contrato no es delegable.
- **Regla**: definimos los **criterios de aceptación** (QUÉ verificar, en comportamiento) — no
  escribimos los tests. **Exigimos** que el dev cubra cada criterio con **tests unitarios + flujos
  end-to-end**; esa cobertura es la Definition of Done y es innegociable.

## 3. Walking Skeleton / Tracer Bullet — de-riscar antes de delegar

Una línea fina end-to-end que atraviesa los componentes reales (UI → lógica → ORM → BD) y "anda".
No es un prototipo desechable: es el esqueleto real sobre el que se construye. El tracer bullet
ataca el **mayor riesgo** primero.

- **Regla**: identificar **el único mecanismo del que depende el diseño** y **trazarlo a la fuente
  de la versión objetivo** (`file:line`) o probarlo con rollback. Ese es el tracer bullet. El
  arquitecto lo entrega como **pseudocódigo detallado + la cita a fuente**, NO como código: como el
  pseudocódigo no corre, es la verificación a fuente lo que de-risquea de verdad.
- **Regla**: definir la primera línea que debe andar end-to-end. El resto del esqueleto cuelga de ahí.
- Por qué: el 80% del riesgo vive en un punto (un hook equivocado, un campo imposible, una API
  externa). Acertar los gotchas periféricos no prueba que el núcleo funcione (cicatriz B-04).

## 4. INVEST + slicing vertical — descomponer bien

Cada tarea/slice debe ser **I**ndependiente, **N**egociable, **V**aliosa, **E**stimable, **S**mall,
**T**esteable. Y sobre todo **vertical**: cada slice entrega funcionalidad usable end-to-end, no
"termina una capa" (no "todos los modelos" y luego "todas las vistas").

- **Regla**: descomponer por valor de negocio vertical, no por capa técnica. Un slice = algo que el
  usuario puede usar y que se puede probar solo.
- **Regla**: si un slice no cabe en la cabeza del dev de un vistazo (no es *Small*), partirlo.

## 5. Definition of Ready (DoR) / Definition of Done (DoD)

- **DoR** (la tarea está lista para delegar): contexto de negocio, ≥2 escenarios, versión objetivo,
  mecanismo central trazado, esqueleto con contratos + pseudocódigo, **criterios de aceptación**,
  contrato fijo vs libertad explícito, <3 preguntas abiertas. Si no se cumple → no se delega.
- **DoD** (el dev terminó): **tests unitarios + flujos end-to-end** que cubren todos los criterios,
  en verde; contrato respetado (firmas intactas); checklist de impacto Odoo cubierto.

## 6. Separación arquitecto / implementador (la línea)

| El ARQUITECTO fija (contrato) | El DEV decide (libertad) |
|-------------------------------|--------------------------|
| Fronteras de módulo, dependencias | Nombres de variables internas |
| Modelos, campos, relaciones | Estructura interna de un método |
| Firmas de método + invariantes | Algoritmo concreto dentro del contrato |
| Puntos de integración / eventos | Orden de operaciones internas |
| Criterios de aceptación (QUÉ verificar) | Los tests que lo verifican (unit + e2e) y hacerlos pasar |
| El pseudocódigo (planos y detalle) | El código real (el cemento) |

Cruzar hacia la derecha = poner cemento = micromanagement (el dev no aprende, copia). Quedarse corto
a la izquierda = el dev redescubre arquitectura y se desvía. El arte es poner la línea en el sitio
correcto: **planos y detalle sí, cemento no**.

### La línea que NO se cruza (reconciliación con dev-task-writer)

`dev-task-writer` prohíbe "dar la solución en código". Este skill **no la cruza**: da **planos**
(esqueleto: firmas, campos, contratos, criterios) y **pseudocódigo** (la lógica en pasos), **nunca
cemento** (Python ejecutable). Ni el **núcleo de riesgo** (tracer bullet) se escribe en código: va
como **pseudocódigo detallado + cita a fuente `file:line` verificada**. Como el pseudocódigo no se
ejecuta, verificar el mecanismo central (grep en fuente o probe con rollback) es obligatorio.

### Filosofía — planos, no cemento

Somos el arquitecto: damos los **planos y el detalle** (esqueleto + pseudocódigo + contratos), **no
ponemos el cemento** (el código real lo escribe el dev). El esqueleto es estructura y firmas (el
contrato); el pseudocódigo describe la lógica en pasos, no en Python ejecutable. El dev es dueño del
CÓMO; el arquitecto fija el QUÉ y la ESTRUCTURA. Toda implementación que respete los planos y cumpla
los criterios es válida por definición.

---

## Referencias externas

- SDD 2025: Thoughtworks, Microsoft Dev Blog, InfoQ, GitHub Spec Kit.
- Walking skeleton: Cockburn; Growing Object-Oriented Software (Freeman & Pryce).
- Tracer bullets: The Pragmatic Programmer (Hunt & Thomas).
- INVEST: Bill Wake / Agile Alliance. Acceptance criteria: Atlassian.
- Design by Contract: Bertrand Meyer (invariantes/pre/post en los docstrings de contrato).
