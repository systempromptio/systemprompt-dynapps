# Template HTML: Agenda para el Cliente

## Instrucciones de Uso

Este template genera el HTML para la agenda que se comparte con el cliente antes de la demo.
Se almacena como nota interna en `mail.message` del lead (no se envia automaticamente).

**Estilos inline obligatorios** (Odoo no procesa CSS externo en emails).

**Placeholders de datos** (del JSON):
- `{client_name}` - Nombre del cliente/partner
- `{demo_date}` - Fecha de la demo (DD/MM/YYYY)
- `{demo_time}` - Hora de la demo (HH:MM)
- `{duration}` - Duracion en minutos
- `{consultant_name}` - Nombre del consultor

**Placeholders narrativos** (generados por el LLM):
- `{intro_paragraph}` - Parrafo introductorio personalizado (lenguaje de negocio)
- `{agenda_items}` - Filas HTML de la tabla con 3-5 items de la agenda
- `{closing_paragraph}` - Parrafo de cierre

**REGLAS**:
- Lenguaje de NEGOCIO, nunca tecnico
- No mencionar "Odoo", "ERP", "modulos" → usar "la solucion", "la plataforma", "el sistema"
- Cada item de agenda describe un RESULTADO, no una funcionalidad
- Duraciones aproximadas por item

---

## Template HTML

```html
<div style="font-family: Arial, Helvetica, sans-serif; color: #333333; max-width: 600px;">

  <!-- HEADER -->
  <div style="background-color: #875A7B; padding: 20px 25px; border-radius: 4px 4px 0 0;">
    <h2 style="color: #ffffff; margin: 0; font-size: 20px;">
      Sesion de Demostracion
    </h2>
    <p style="color: #e8d5e0; margin: 5px 0 0 0; font-size: 14px;">
      {client_name} &mdash; {demo_date}
    </p>
  </div>

  <div style="padding: 20px 25px; border: 1px solid #e0e0e0; border-top: none; border-radius: 0 0 4px 4px;">

    <!-- INTRO -->
    <p style="font-size: 14px; line-height: 1.6; color: #555555;">
      {intro_paragraph}
    </p>

    <!-- DETALLES -->
    <table style="width: 100%; margin: 15px 0; font-size: 13px;">
      <tr>
        <td style="padding: 4px 0; color: #875A7B; font-weight: bold; width: 120px;">Fecha:</td>
        <td style="padding: 4px 0;">{demo_date}</td>
      </tr>
      <tr>
        <td style="padding: 4px 0; color: #875A7B; font-weight: bold;">Hora:</td>
        <td style="padding: 4px 0;">{demo_time}</td>
      </tr>
      <tr>
        <td style="padding: 4px 0; color: #875A7B; font-weight: bold;">Duracion:</td>
        <td style="padding: 4px 0;">{duration} minutos aprox.</td>
      </tr>
      <tr>
        <td style="padding: 4px 0; color: #875A7B; font-weight: bold;">Presentador:</td>
        <td style="padding: 4px 0;">{consultant_name}</td>
      </tr>
    </table>

    <!-- AGENDA -->
    <h3 style="color: #875A7B; font-size: 16px; border-bottom: 2px solid #875A7B; padding-bottom: 8px;">
      Agenda
    </h3>

    <table style="width: 100%; border-collapse: collapse; margin: 15px 0; font-size: 13px;">
      <thead>
        <tr style="background-color: #f5f0f3;">
          <th style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0; color: #875A7B; width: 30px;">#</th>
          <th style="text-align: left; padding: 8px 12px; border: 1px solid #e0e0e0; color: #875A7B;">Tema</th>
          <th style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0; color: #875A7B; width: 70px;">Tiempo</th>
        </tr>
      </thead>
      <tbody>
        {agenda_items}
      </tbody>
    </table>

    <!-- CIERRE -->
    <p style="font-size: 14px; line-height: 1.6; color: #555555;">
      {closing_paragraph}
    </p>

  </div>

  <!-- FOOTER -->
  <div style="text-align: center; padding: 15px; font-size: 11px; color: #999999;">
    <strong>Indaws</strong> &mdash; Odoo Gold Partner
  </div>

</div>
```

---

## Formato de Filas de Agenda

### agenda_items (una por punto de la agenda)

```html
<tr>
  <td style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0; font-weight: bold; color: #875A7B;">{n}</td>
  <td style="padding: 8px 12px; border: 1px solid #e0e0e0;">{item_description}</td>
  <td style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0;">{item_duration} min</td>
</tr>
```

### Ejemplo de items (lenguaje de negocio)

| # | Tema (MAL - tecnico) | Tema (BIEN - negocio) |
|---|---------------------|----------------------|
| 1 | "Modulo CRM y pipeline" | "Gestion comercial y seguimiento de oportunidades" |
| 2 | "Modulo de Ventas (sale.order)" | "Creacion y envio de presupuestos profesionales" |
| 3 | "Inventario y stock" | "Control de existencias y movimientos de almacen" |
| 4 | "Contabilidad (account.move)" | "Facturacion y cobros integrados" |
| 5 | "Q&A" | "Preguntas y proximos pasos" |

### Reglas de Generacion

1. **3-5 items maximo** (incluyendo apertura y cierre si se muestran)
2. **Lenguaje de resultados**: "Como agilizar...", "Control de...", "Automatizacion de..."
3. **NO mencionar nombres tecnicos**: No "CRM", no "sale.order", no "modulo"
4. **Tiempos aproximados**: Redondear a 5 o 10 minutos
5. **Ultimo item siempre**: "Preguntas y proximos pasos"
