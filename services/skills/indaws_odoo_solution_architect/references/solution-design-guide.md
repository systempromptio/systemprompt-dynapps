# Guía de Diseño de Solución (Step 2)

Cómo pasar de una necesidad a un diseño delegable. Orden fijo — cada paso alimenta al siguiente.

---

## 1. Fijar el mecanismo central (tracer bullet) — PRIMERO

Antes de dibujar módulos, encontrar **el único punto del que depende que la solución sea posible**:
el hook, el campo, el constraint, la API. Trazarlo a la fuente de la versión objetivo (`file:line`)
o probarlo con rollback (`odoo-flow-tester`).

- Preguntas: ¿existe el hook que asumo? ¿el campo admite el valor que necesito? ¿el compute se
  dispara cuando creo que se dispara? ¿la restricción de BD me deja hacer esto?
- **Si el mecanismo central no se sostiene, todo el diseño cae.** No seguir hasta confirmarlo.
- Cicatriz B-04: se diseñó sobre `_fsm_create_sale_order_line` (hook equivocado) y `sale_line_id` a
  un bien (imposible) sin abrir la fuente. Los gotchas periféricos estaban bien; el núcleo, no.

## 2. Fronteras de módulo (SRP)

- **1 módulo = 1 problema de negocio.** Si la descripción lleva "y" → probablemente dos módulos.
- **Nunca tocar core**: extender por herencia (`_inherit`) en módulo propio.
- **Bridge module** si A y B son independientes pero deben interactuar: módulo C (`auto_install=True`)
  que solo contiene el puente. No meter la dependencia cruzada dentro de A ni de B.
- ¿La funcionalidad sirve a >1 cliente? → candidato a Core horizontal, no custom específico.

## 3. Elegir el patrón

| Necesidad | Patrón |
|-----------|--------|
| Añadir campos/lógica a un modelo existente | Herencia `_inherit` |
| Modelo que "es un" otro + datos propios | Delegación `_inherits` |
| Comportamiento reutilizable en varios modelos | Mixin `AbstractModel` |
| Flujo temporal con UI (asistente) | Wizard `TransientModel` |
| Conectar dos módulos independientes | Bridge module |
| Cálculo derivado | Campo `computed` (¿`store`? ver impacto en registros existentes) |

## 4. Descomponer en slices verticales (INVEST)

- Partir por **valor end-to-end**, no por capa. Mal: "slice 1 = modelos, slice 2 = vistas". Bien:
  "slice 1 = crear y ver el registro con su campo mínimo; slice 2 = el cálculo; slice 3 = el report".
- Cada slice: independiente, testeable solo, pequeño. Si no cabe de un vistazo → partir.
- Ordenar por **riesgo y valor**: primero el slice que contiene el tracer bullet.

## 5. Definir el walking skeleton

La primera línea end-to-end que debe andar: de la acción del usuario, por la lógica, al ORM y de
vuelta. Es el esqueleto real, no un prototipo a tirar. Todo lo demás cuelga de esa línea.

## 6. ADR (Architecture Decision Record)

Documentar en `templates/solution-design.md`:
- **Decisión**: qué se construye y con qué patrón.
- **Alternativas descartadas**: qué se consideró y por qué NO (esto evita que el dev "mejore" el
  diseño reintroduciendo lo ya descartado).
- **Consecuencias**: qué se gana, qué deuda/limitación se asume, qué queda fuera.
- **One-way vs two-way door**: si es irreversible (cambio de modelo de datos, migración) → analizar
  a fondo; si es reversible → decidir rápido y no bloquear.

---

## Checklist de salida de Step 2 (antes de generar esqueleto)

- [ ] Mecanismo central trazado a `file:line` (o probe con rollback).
- [ ] Fronteras de módulo definidas (SRP, core intacto, bridge si aplica).
- [ ] Patrón elegido y justificado.
- [ ] Slices verticales listados y ordenados por riesgo/valor.
- [ ] Walking skeleton identificado (la primera línea end-to-end).
- [ ] ADR con alternativas descartadas.
- [ ] Deps mínimas verificadas (existen y aportan).
