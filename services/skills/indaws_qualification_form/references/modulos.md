# Modulos Odoo — Cualificacion

Lista de 15 modulos a presentar al SDR en el Paso 5 del skill `qualification-form`, agrupados en 4 tandas para respetar el limite de 4 opciones por llamada de `AskUserQuestion`.

**No alterar nombres** — se renderizan tal cual en el HTML final como chips.

---

## Tanda 1 — Comercial (4 opciones)

- `CRM`
- `Ventas`
- `Marketing / Emailing`
- `eCommerce`

## Tanda 2 — Operaciones (4 opciones)

- `Compras`
- `Inventario`
- `Fabricacion (MRP)`
- `Proyectos`

## Tanda 3 — Finanzas & RRHH (4 opciones)

- `Contabilidad`
- `Facturacion`
- `Recursos Humanos`
- `Control Horario`

## Tanda 4 — Customer-facing & otros (3 opciones)

- `SAT (Servicio Posventa)`
- `Suscripciones`
- `Punto de Venta (POS)`

---

## Notas para el skill

- Llamar a `AskUserQuestion` con `multiSelect=true` una vez por tanda, con header `"Modulos — <nombre de la tanda>"`.
- Acumular las selecciones de las 4 tandas en una unica lista `modulos` en el orden en que se seleccionaron.
- En el HTML final, cada modulo se renderiza como chip de acento (Sunset Tangerine `#FF7E23` sobre Midnight Denim `#050543`).
- Si el SDR selecciona 0 modulos, mostrar `—` en el resumen.
- Despues de la 4ª tanda, pedir notas opcionales: desarrollos especiales, integraciones externas, requisitos especificos.
