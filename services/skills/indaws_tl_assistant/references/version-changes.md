# Cambios Clave entre Versiones (Impacto Decisiones TL)

> Detalle completo de API: consultar `/odoo-version-api-reference`

> ⚠️ **Esta página es orientación de TL, NO fuente.** Las líneas de abajo **no llevan cita** y su
> propósito es dar conversación con el cliente, no decidir sintaxis. **Antes de comprometer nada en
> una migración, comprueba el punto concreto en `odoo-version-api-reference` o en `/opt/odoo`.**
> Repasada el 2026-08-18 y **una entrada estaba un major tarde** (OWL 2, ver abajo): asume que
> puede haber más.

## v17 (Oct 2023)
- `name_get()` deprecated → `_compute_display_name()`
- `_read_group()` cambio firma completo
- ~~OWL 2 obligatorio~~ → **eso es de v16, no de v17** (corregido 2026-08-18): los imports de
  `@odoo/owl` en `web/static/src` pasan de **0 en v15 a 261 en v16** (287 en v17). Lo que v17 aporta
  es continuidad, no la ruptura. **La conversación de migración sobre OWL toca en v15→v16.**
- Nuevo sistema de assets
- Vistas lista mejoradas (agrupamiento, multi-edición) → puede eliminar vistas custom

## v18 (Oct 2024)
- Formularios simplificados, nuevos componentes UI
- Studio más potente → más cosas sin código
- Reporting engine actualizado
- Mejoras rendimiento core → pueden hacer innecesarias optimizaciones custom

## v19 (Oct 2025)
- JSON2 protocol (junto a JSON-RPC) → afecta integraciones externas
- Nuevos patrones vistas
- Sistema permisos actualizado → revisar seguridad custom
- ORM más eficiente

## Impacto por Área

| Área | v17 | v18 | v19 |
|------|-----|-----|-----|
| Python | name_get, _read_group | Nuevos helpers | ORM optimizado |
| Vistas | Nuevos atributos listas | Forms simplificados | Nuevos patrones |
| JS | (OWL 2 es **v16**) | Nuevos componentes | — |
| API/Integración | — | — | JSON2 |
| Seguridad | — | — | Permisos actualizado |
| Studio | Base | Más potente | — |

## Pregunta TL por módulo en cada migración

1. ¿Funcionalidad ahora en estándar? → **Eliminar módulo**
2. ¿Se resuelve con Studio? → **Migrar a Studio** (no es un limite de v18: campos computados,
   Python en automatizaciones y approvals existen desde v14 — ver `odoo-classification-guide`)
3. ¿Usa APIs deprecated? → **Migrar obligatoriamente**
4. ¿Tiene tests? → **Momento de añadirlos**
5. ¿Se usa realmente? → **Si no, eliminar**
