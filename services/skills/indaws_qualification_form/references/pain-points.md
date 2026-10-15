# Pain Points — Cualificacion Odoo

Los 10 pain points exactos a usar en el Paso 4 del skill `qualification-form`. **No alterar texto ni emojis** — son la fuente de la verdad y se renderizan tal cual en el HTML final.

`AskUserQuestion` solo permite 4 opciones por llamada, por lo que se reparten en **3 tandas** logicamente agrupadas. El `id` interno es solo para referencia; en el HTML final usar `emoji + title`.

---

## Tanda 1 — Datos & procesos (4 opciones)

| ID | Emoji | Title | Description |
|---|---|---|---|
| `dispersos` | 🗂️ | Datos dispersos en multiples herramientas | Usan Excel, un sistema de facturacion, otro de inventario y uno mas de RRHH. Nadie tiene una vision unificada del negocio y se pierde tiempo reconciliando informacion. |
| `manuales` | 🔁 | Procesos manuales que generan errores | Ingresar el mismo dato en 2 o 3 sistemas distintos es pan de cada dia. Eso significa errores, retrasos y empleados haciendo trabajo de bajo valor. |
| `reportes` | 📊 | Reportes que llegan tarde o son poco confiables | Los directivos toman decisiones con informacion de la semana pasada, o peor, no confian en los numeros porque cada area tiene "su version" de la realidad. |
| `visibilidad` | 🚫 | Sin visibilidad del estado real de un cliente o pedido | Ventas no sabe si el pedido salio, finanzas no sabe si fue cobrado, y el cliente llama preguntando. Nadie tiene el hilo completo. |

---

## Tanda 2 — Tecnologia & coste (4 opciones)

| ID | Emoji | Title | Description |
|---|---|---|---|
| `caro` | 💸 | ERP actual demasiado caro para lo que ofrece | Pagan licencias altas de SAP, Microsoft Dynamics u Oracle, pero solo usan el 30% de las funcionalidades. El ROI no se justifica. |
| `escala` | 🧱 | Sistema actual que ya no escala | Empezaron pequeños y su sistema "de toda la vida" ya no soporta el volumen de operaciones, usuarios o la complejidad del negocio actual. |
| `integracion` | 🔌 | Falta de integracion con herramientas modernas | Su ERP no se conecta con su e-commerce, su CRM, su banco, o sus marketplaces. Todo requiere trabajo manual o desarrollos costosos. |
| `dependencia` | 🧑‍💻 | Dependencia total del proveedor o del area de TI | Cualquier cambio pequeño, como un nuevo campo o un reporte distinto, requiere meses de espera y miles de dolares en consultoria. |

---

## Tanda 3 — Operaciones & experiencia (2 opciones)

| ID | Emoji | Title | Description |
|---|---|---|---|
| `inventario` | 📦 | Inventario descontrolado | No saben con precision que tienen en almacen, donde esta, o cuando se necesita reponer. Eso genera quiebres de stock o sobreinventario. |
| `experiencia` | 👥 | Mala experiencia del equipo de ventas y operaciones | El sistema es lento, poco intuitivo y los equipos lo evitan o trabajan por fuera de el, generando procesos "en la sombra". |

---

## Notas para el skill

- Pasa el campo `description` de cada opcion en la llamada a `AskUserQuestion` para que el SDR vea el contexto.
- En el HTML final, cada pain point seleccionado se renderiza como chip: `<span class="chip">{emoji} {title}</span>`.
- Si el SDR no selecciona ninguno en una tanda, simplemente continua con la siguiente. No es obligatorio seleccionar al menos uno.
- En el resumen plano (Paso 7), lista los pain points seleccionados en una sola linea separados por ` · `.
