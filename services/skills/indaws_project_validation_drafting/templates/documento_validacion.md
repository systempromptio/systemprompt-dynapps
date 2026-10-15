# Template: Documento de Validacion Completo (Modo DOCUMENTAR)

Este template define la estructura del documento Markdown que genera el modo DOCUMENTAR.
El documento se convierte a HTML con estilos inline para actualizar `x_description` del `x_client_request`.

---

## Estructura del Documento

```markdown
# Validacion: {titulo}

**Proyecto**: {nombre_proyecto}
**Cliente**: {nombre_cliente}
**Fecha**: {fecha_generacion}
**Consultor**: Indaws Business Solutions S.L.

---

## 1. Que se ha entregado

{descripcion_en_lenguaje_negocio}

Resumen de lo implementado, redactado como resultados de negocio.
Extraido de los checklists completados de la tarea, traducidos.

Ejemplo:
> Hemos configurado la gestion de ventas de {nombre_cliente} en Odoo.
> Ahora podeis crear presupuestos, enviarlos por email a vuestros clientes,
> convertirlos en pedidos y generar albaranes y facturas de forma automatica.
>
> En concreto:
> - Vuestras 3 tarifas (mayorista, minorista, VIP) estan configuradas con los descuentos acordados
> - Los presupuestos se generan con el formato de documento aprobado
> - El equipo comercial (Maria, Pedro, Luis) tiene acceso a sus oportunidades y presupuestos

---

## 2. Como funciona ahora

{flujo_perspectiva_usuario}

Describir paso a paso lo que el usuario final hara en su dia a dia.
Usar nombres reales del entorno del cliente.

Ejemplo:
> 1. Maria (comercial zona norte) entra en Odoo y crea un nuevo presupuesto
> 2. Selecciona el cliente "Hotel Plaza Mayor" de la lista
> 3. Anade los productos: "Sofa Modelo Valencia 3 plazas" (tarifa mayorista: 1.250 EUR)
> 4. El sistema aplica automaticamente el 10% de descuento por volumen (pedido > 5.000 EUR)
> 5. Maria envia el presupuesto por email directamente desde Odoo
> 6. Cuando el cliente acepta, Maria confirma el pedido con un clic
> 7. Se genera automaticamente el albaran de salida en el almacen Central

---

## 3. Criterios de aceptacion

Os pedimos que reviseis los siguientes puntos y nos confirmeis que todo es correcto:

{lista_criterios}

Cada criterio debe ser verificable por el cliente sin ayuda tecnica.

Ejemplo:
> - [ ] Los presupuestos muestran los precios correctos para cada tarifa
> - [ ] Al confirmar un pedido, se genera el albaran automaticamente
> - [ ] Los descuentos por volumen se aplican correctamente (10% a partir de 5.000 EUR)
> - [ ] El email de envio de presupuesto llega con el formato y adjunto correctos
> - [ ] Cada comercial solo ve sus propios presupuestos y pedidos
> - [ ] Los informes de ventas muestran los datos esperados

---

## 4. Evidencia visual

{screenshots_o_placeholders}

Si hay credenciales del Odoo del cliente, incluir placeholders:
> [SCREENSHOT: Vista de presupuesto con tarifa mayorista aplicada]
> [SCREENSHOT: Email de presupuesto recibido por el cliente]
> [SCREENSHOT: Albaran generado automaticamente tras confirmar pedido]

Si NO hay credenciales, omitir esta seccion.

---

## 5. Limitaciones y aclaraciones

{limitaciones}

Ser transparente sobre lo que NO esta incluido.

Ejemplo:
> - La integracion con vuestra pasarela de pago se validara por separado (validacion especifica)
> - Los informes personalizados de ventas por zona se implementaran en la fase 2
> - Los descuentos promocionales temporales no estan incluidos en esta fase

---

## 6. Aprobacion

Para que podamos continuar con las siguientes fases del proyecto, necesitamos vuestra conformidad con lo implementado en esta validacion.

**Resultado de la validacion:**

- [ ] **Aprobado** — Todo funciona segun lo esperado
- [ ] **Aprobado con observaciones** — Funciona pero hay puntos a ajustar (indicar abajo)
- [ ] **No aprobado** — Hay aspectos que no cumplen con lo esperado (indicar motivos abajo)

**Observaciones:**

_______________________________________________________

**Nombre y cargo**: ___________________________________
**Fecha**: ____________________________________________
```

---

## Reglas de generacion

### Seccion 1 (Que se ha entregado)
- Leer checklists completados de la tarea (`state = 'done'` en `xx.checklist.item`)
- Traducir cada item del checklist a un resultado de negocio
- Agrupar por funcionalidad, no por item tecnico

### Seccion 2 (Como funciona ahora)
- Crear un escenario realista con datos del cliente
- Maximo 10 pasos por flujo
- Si hay varios flujos, separarlos con subtitulos

### Seccion 3 (Criterios de aceptacion)
- Minimo 5, maximo 12 criterios por validacion
- Cada criterio debe poder responderse con Si/No
- Incluir datos concretos (cantidades, nombres, precios)

### Seccion 4 (Evidencia visual)
- Solo si hay credenciales del Odoo del cliente
- Maximo 5 screenshots por validacion
- Cada placeholder describe exactamente que se debe capturar

### Seccion 5 (Limitaciones)
- Solo mencionar lo relevante para esta validacion
- No listar todo lo que falta en el proyecto, solo lo relacionado

### Seccion 6 (Aprobacion)
- No modificar la estructura de aprobacion — es estandar
