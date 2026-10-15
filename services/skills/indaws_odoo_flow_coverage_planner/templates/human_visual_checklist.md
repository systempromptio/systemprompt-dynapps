# Checklist de validación VISUAL (humano) — {{instance_name}}

> Estas validaciones se dejan FUERA de la IA a propósito: son visuales/maquetación, mucho más
> rápidas y baratas si las hace una persona. La IA ya validó por API los DATOS y estados.
> Aquí solo queda mirar el ASPECTO.

- Versión Odoo: {{target_version}} · Entorno: {{url}}
- Tiempo estimado: ~{{n}} min · Ítems: {{count}}

> **Cada ítem indica SIEMPRE 3 cosas:** `Ruta` (cómo llegar: menú o URL exacta) · `Cómo probar`
> (pasos concretos para reproducirlo) · `Criterio` (qué debe verse para dar OK).
> Marca el resultado: [x] OK / [!] problema (anota).

---

## [ ] 1. {{Título corto del ítem}}  ·  módulo: {{módulo}}
- **Ruta:** {{menú exacto, p.ej. `Ventas → Pedidos → <pedido ejemplo>` o URL `http://.../odoo/sales`}}
- **Cómo probar:** {{pasos numerados, p.ej. "1) Abre el pedido [TEST-...]; 2) botón Imprimir → Presupuesto; 3) abre el PDF"}}
- **Criterio de aceptación:** {{qué debe verse, p.ej. "la columna 'Pos.' queda alineada y no se solapa"}}
- **Dato de ejemplo:** {{registro/URL concretos si los hay; los [TEST-...] del flow-tester o uno real}}

## [ ] 2. {{...}}  ·  módulo: {{...}}
- **Ruta:** {{...}}
- **Cómo probar:** {{...}}
- **Criterio de aceptación:** {{...}}
- **Dato de ejemplo:** {{...}}

---

## Resultado
- OK: {{n_ok}} / {{count}}
- Incidencias visuales (a corregir): {{lista}}

> Nota de autoría: NUNCA dejar `Ruta`/`Cómo probar` en blanco o genérico. Si no se conoce la ruta
> exacta, derivarla del menú del modelo (`ir.ui.menu`) o de la acción asociada, o indicar la URL
> `/odoo/action-<id>`. El objetivo es que el humano tarde segundos en llegar y saber qué hacer.
