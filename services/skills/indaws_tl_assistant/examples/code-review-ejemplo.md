# Code Review — PR #452: indaws_sale_rappel

**Módulo**: indaws_sale_rappel | **Dev**: Mid (2 años Odoo) | **TL**: Toni Hermosilla
**Odoo**: 18.0 | **Fecha**: 2026-03-15

---

## Resumen

PR que añade cálculo automático de rappel en líneas de venta según configuración del cliente.

## Hallazgos

| # | Severidad | Área | Hallazgo | Sugerencia |
|---|-----------|------|----------|------------|
| 1 | **Blocker** | Seguridad | `sudo()` en `_compute_rappel_amount` expone costes a usuarios sin permiso | Usar `with_company()` en lugar de `sudo()`. El cálculo debe respetar reglas de acceso del grupo "Ver costes" |
| 2 | **Major** | Rendimiento | `search()` dentro de `_compute_rappel_amount` genera N+1 queries en lista de pedidos | Prefetch: mover el search fuera del loop, agrupar por partner_id y mapear |
| 3 | **Minor** | Python | `if rappel_config != False:` — comparación explícita con False | Usar `if rappel_config:` — más pythónico y cubre None/False/empty |
| 4 | **Info** | Tests | Solo testea caso con rappel activo, falta caso sin rappel y caso con rappel expirado | Añadir test `test_no_rappel_configured` y `test_rappel_expired` |
| 5 | **Info** | Buenas prácticas | Buen uso de `@api.depends` con campos correctos. Naming consistente con convención Core | Reconocimiento: buen trabajo en estructura y naming |

## Veredicto

**Devolver al dev** — 1 Blocker (seguridad) + 1 Major (rendimiento).

Comunicar al dev: "La lógica de negocio está bien planteada. Hay dos temas que resolver antes de merge: el sudo() es un problema de seguridad (explico por qué en el comentario) y el search dentro del compute va a ser lento en pedidos con muchas líneas. ¿Has considerado prefetch por partner? Mira cómo lo hace `sale_margin` para el patrón."

---

## Notas para el TL

- El dev mostró buen criterio en la estructura del módulo — dar feedback positivo explícito
- El patrón N+1 es recurrente en este dev — considerar sesión de mentoring sobre prefetch
- El tema de `sudo()` es un learning moment: explicar el impacto de seguridad, no solo el fix
