# Verificación de Contrato (Step 3)

Revisamos la entrega del dev **contra el contrato del MD** que generó `odoo-solution-architect`, no
contra el gusto personal. El dev es libre en el CÓMO; el contrato fija el QUÉ y la ESTRUCTURA.

---

## 1. Firmas y campos `# CONTRACT` intactos

- Cada método/campo marcado `# CONTRACT` en el esqueleto: **mismo nombre, parámetros, tipo de
  retorno, `store`, `depends`**. Un cambio aquí rompe a quien depende de la interfaz.
- Si el dev cambió una firma:
  - ¿Sin motivo? → **correcciones**: restaurar el contrato.
  - ¿Con razón técnica válida? → **NO se aprueba por la vía rápida**: vuelve a
    `odoo-solution-architect` para actualizar el contrato y a quien dependa de él.

## 2. Fronteras de módulo y dependencias

- `__manifest__.py`: **sin deps nuevas** que no estén en el contrato. Una dep de más arrastra cadena.
- **Core intacto**: la funcionalidad se añadió por herencia (`_inherit`), no modificando el core.
- El módulo sigue haciendo **una sola cosa** (SRP) — no se coló funcionalidad no pedida (scope creep).

## 3. Pseudocódigo → código (mismo QUÉ)

- El cuerpo implementa lo que el pseudocódigo `# PSEUDO` describía: mismas reglas, mismos invariantes,
  mismos casos borde (p.ej. "precio 0 → 0.0, no div/0").
- El **CÓMO es del dev** (algoritmo, helpers, nombres locales): no marcar como error algo que sea
  simplemente una implementación distinta y correcta.
- El **núcleo de riesgo** (tracer bullet): el dev lo construyó sobre el mecanismo verificado que
  citamos (`file:line`), no inventó otro. Si cambió el mecanismo → revisar con lupa.

## 4. Puntos de integración

- `super()` respetado donde el contrato lo exigía (no se pisó el comportamiento core).
- Eventos/hooks/señales que el contrato declaraba: presentes y en el punto correcto.
- Lo que otros módulos/tareas esperaban de esta interfaz sigue cumpliéndose.

## 5. Contrato-fijo vs libertad (no invadir)

Antes de marcar un finding, preguntar: **¿esto es contrato o libertad del dev?**

| Es finding (corrección) | NO es finding (su libertad) |
|-------------------------|------------------------------|
| Cambió una firma `# CONTRACT` | Nombró distinto una variable local |
| No cubre un criterio de aceptación | Ordenó las operaciones a su manera |
| Añadió dep no acordada | Extrajo un helper privado |
| Rompió un punto de integración | Optimizó sin cambiar la interfaz |

Invadir la columna derecha = micromanagement; el dev deja de aprender y de aportar.

---

## Salida

Lista de findings de contrato, cada uno con: **qué incumple**, **qué cláusula del contrato/criterio**,
y **si es fallo del dev o del contrato** (este último escala al arquitecto). Alimenta el Step 7 (decisión).
