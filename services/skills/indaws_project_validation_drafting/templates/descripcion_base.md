# Template: Descripcion Base para x_client_request (Modo PLANIFICAR)

Este template genera el HTML para `x_description` del `x_client_request` en modo PLANIFICAR.
Es una descripcion breve que anticipa al cliente lo que tendra que validar.

---

## Estructura HTML

```html
<div style="font-family: Verdana, sans-serif; font-size: 14px; color: #333; line-height: 1.6;">

  <p style="font-size: 16px; font-weight: bold; color: #1C265D; margin-bottom: 16px;">
    {titulo_validacion}
  </p>

  <p><strong>Que vais a validar:</strong></p>
  <p>{descripcion_en_lenguaje_negocio}</p>

  <p style="margin-top: 12px;"><strong>Puntos clave que revisaremos juntos:</strong></p>
  <ul>
    {criterios_preliminares}
    <!-- Cada criterio como <li>{criterio}</li> -->
  </ul>

  <p style="margin-top: 16px; padding: 12px; background: #f0f4ff; border-left: 4px solid #6B68FA; font-size: 13px; color: #555;">
    Cuando la implementacion de esta parte este completada, os enviaremos un documento detallado con los resultados y los puntos concretos que necesitaremos que reviseis.
  </p>

</div>
```

---

## Reglas de generacion

### Para tipo APP

- `{descripcion}`: Resumen del analisis de esa app en 2-3 frases de negocio
- `{criterios}`: 3-5 puntos extraidos del analisis (funcionalidades clave que el cliente espera)
- Ejemplo para Ventas: "Validareis que la gestion de presupuestos, pedidos y tarifas funciona segun vuestras necesidades"

### Para tipo MIGRACION

- `{descripcion}`: Que datos se van a migrar y de donde vienen
- `{criterios}`: Entidades a migrar (productos, clientes, facturas) con volumenes aproximados
- Ejemplo: "Validareis que los 2.347 productos y 850 clientes migrados desde vuestro sistema anterior aparecen correctamente en Odoo"

### Para tipo DESARROLLO

- `{descripcion}`: Que hace el desarrollo/integracion en terminos de negocio
- `{criterios}`: Resultados esperados que el cliente podra verificar
- Ejemplo: "Validareis que la conexion con vuestra pasarela de pago procesa correctamente los cobros online"

### Para tipo PREPRODUCCION

- `{descripcion}`: Validacion global de que el sistema esta listo para el dia a dia
- `{criterios}`: Flujos completos end-to-end que el cliente recorrera
- Ejemplo: "Validareis que podeis operar con Odoo en vuestro dia a dia: desde crear un presupuesto hasta cobrar la factura"
