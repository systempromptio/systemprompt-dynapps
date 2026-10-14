# Intake — Validar peticion antes de redactar

## Para que sirve

Antes de redactar una tarea para dev, la peticion debe contener la informacion minima.
Sin esto, el redactor itera con el solicitante y se pierde tiempo.

Esta validacion la ejecuta `dev-task-writer` en Step 0. Si falta algo critico → devolver
template vacio al consultor/PM/TL solicitante. NO adivinar.

## Template

```markdown
## Problema
{Descripcion funcional. Que hace el usuario hoy y que deberia poder hacer.
Hablar de negocio, NO de codigo. No proponer solucion tecnica.}

## Version Odoo
{14.0 / 16.0 / 17.0 / 18.0 / 19.0}

## Usuarios afectados
{Quien usa esto: comerciales, almacen, contabilidad, portal, etc.
Si es posible, indicar volumen: "20 comerciales", "3 operarios".}

## Comportamiento esperado
{Escenarios concretos. Minimo 2, idealmente 3-5:
- Cuando el usuario hace X, debe pasar Y
- Si Z esta vacio, el comportamiento es W
- En el caso de A, no debe permitir B}

## Contexto adicional (si aplica)
- Modulos Indaws Core instalados: {indaws_sale, indaws_account, etc.}
- Integraciones activas: {SEUR, factura-e, pasarela pago, etc.}
- Hay datos existentes que migrar/transformar?
- Volumen esperado: {n registros, frecuencia de uso}

## Lo que NO quiero
{Funcionalidad que Odoo estandar ya deberia cubrir.
Overengineering que evitar. Casos que quedan fuera del alcance.}

## Prioridad
{Critico (bloquea operacion) / Alto (impacta eficiencia) / Medio / Bajo}

## Referencias
{Links a tickets, tareas Odoo, capturas de pantalla, documentos.
Si hay un modulo v13/v16 que hacia algo similar, indicar nombre.}
```

## Por que cada seccion importa

| Seccion | Skill que lo consume | Que pasa si falta |
|---------|---------------------|-------------------|
| **Problema** (negocio) | `dev-assessment` evalua Standard/Studio/Custom | Se desarrolla algo que ya existe |
| **Version** | `odoo-version-api-reference` carga API correcta | Codigo con API deprecated |
| **Usuarios** | TL decide permisos y UX | Modulo sin grupos, campos visibles a quien no debe |
| **Comportamiento esperado** | `odoo-test-generator` → tests directos | Tests vagos, bugs en produccion |
| **Modulos Core** | `indaws-core-developer` carga dependencias | Reinventa lo que indaws_core ya hace |
| **Lo que NO quiero** | `dev-assessment` descarta antes de codificar | Scope creep, horas perdidas |
| **Prioridad** | TL planifica en workload | Se trabaja en lo urgente tarde |
| **Referencias** | TL verifica coherencia | Se malinterpreta la necesidad |

## Ejemplo rellenado

```markdown
## Problema
La politica de facturacion en ventas se define a nivel de producto.
Cuando puntualmente necesitamos facturar un pedido completo antes de
entregar, tenemos que cambiar el producto (afectando otros pedidos)
o usar anticipos (confuso contablemente).

## Version Odoo
18.0

## Usuarios afectados
Responsables de ventas (3 personas). Caso puntual, no diario.

## Comportamiento esperado
- Cuando el campo esta vacio: comportamiento estandar de Odoo
- Cuando selecciono "Cantidades pedidas": puedo facturar el pedido
  completo sin haber entregado
- Cuando lo limpio: vuelve al comportamiento del producto
- El cambio queda registrado en el chatter (auditoria)

## Contexto adicional
- No depende de indaws_core
- Hereda multi-company de sale.order
- Se necesita lo mismo para compras (modulo hermano)

## Lo que NO quiero
- No cambiar la politica del producto (afecta otros pedidos)
- No usar anticipos como workaround
- No anadir wizard: un campo en "Otra informacion" es suficiente

## Prioridad
Medio — no bloquea, pero genera trabajo manual frecuente.

## Referencias
- Odoo estandar: `sale.advance.payment.inv` (wizard anticipos)
- Campos: `product.template.invoice_policy`, `product.template.purchase_method`
```

## Checklist al recibir una peticion

- [ ] **Problema** descrito en lenguaje de negocio (no tecnico)
- [ ] **Version** Odoo explicita
- [ ] **Al menos 2 escenarios** de comportamiento esperado
- [ ] **Prioridad** indicada
- [ ] No propone solucion tecnica (eso lo decide el TL)

**Si falta algo critico** → devolver template vacio al solicitante.
No invertir tiempo en adivinar lo que deberia haber dicho.

## Anti-patrones en peticiones

| Anti-patron | Problema | Que pedir |
|-------------|----------|-----------|
| "Anadir campo X al modelo Y" | Propone solucion, no problema | "Que necesita hacer el usuario que hoy no puede?" |
| "Hacer como en v13" | Asume que la v13 era correcta | "Que comportamiento especifico necesitas?" |
| "Que funcione bien" | No verificable | "Dame 3 escenarios con resultado esperado" |
| "Es urgente" sin contexto | No priorizable | "Que operacion bloquea si no se hace?" |
| Ticket + "arreglar esto" | Sin analisis | "Cual es el comportamiento correcto?" |
