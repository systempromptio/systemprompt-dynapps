# Template: Entrada de Acta de Reunion

Template HTML para una entrada individual de acta. Se inserta dentro del articulo acumulativo "Actas de Reunion".

Todos los estilos son inline (obligatorio para Odoo Knowledge).

---

## HTML de una Entrada de Acta

```html
<div data-acta-fecha="{YYYY-MM-DD}" data-acta-tipo="{tipo}" style="margin-bottom: 20px;">

  <!-- HEADER -->
  <div style="display: flex; align-items: center; gap: 12px; margin-bottom: 16px; flex-wrap: wrap;">
    <span style="display: inline-block; padding: 4px 12px; border-radius: 4px; font-size: 12px; font-weight: 600; background: {badge_bg}; color: {badge_text}; text-transform: uppercase; letter-spacing: 0.5px;">
      {TIPO}
    </span>
    <span style="font-size: 18px; font-weight: 600; color: #1C265D;">
      {DD/MM/YYYY}
    </span>
    {si_duracion:}
    <span style="font-size: 14px; color: #666;">
      ({duracion})
    </span>
  </div>

  <!-- ASISTENTES -->
  <div style="margin-bottom: 20px;">
    <h3 style="font-size: 14px; color: #6B68FA; margin: 0 0 8px 0; text-transform: uppercase; letter-spacing: 0.5px;">Asistentes</h3>
    <table style="border-collapse: collapse; width: 100%; font-size: 14px;">
      <thead>
        <tr style="background: #f8f9fa;">
          <th style="padding: 6px 12px; border: 1px solid #dee2e6; text-align: left; font-weight: 600;">Nombre</th>
          <th style="padding: 6px 12px; border: 1px solid #dee2e6; text-align: left; font-weight: 600;">Rol</th>
          <th style="padding: 6px 12px; border: 1px solid #dee2e6; text-align: left; font-weight: 600;">Empresa</th>
        </tr>
      </thead>
      <tbody>
        {por_cada_asistente:}
        <tr>
          <td style="padding: 6px 12px; border: 1px solid #dee2e6;">{nombre}</td>
          <td style="padding: 6px 12px; border: 1px solid #dee2e6;">{rol}</td>
          <td style="padding: 6px 12px; border: 1px solid #dee2e6;">{empresa}</td>
        </tr>
      </tbody>
    </table>
  </div>

  <!-- RESUMEN EJECUTIVO -->
  <div style="margin-bottom: 20px;">
    <h3 style="font-size: 14px; color: #6B68FA; margin: 0 0 8px 0; text-transform: uppercase; letter-spacing: 0.5px;">Resumen</h3>
    <ul style="margin: 0; padding-left: 20px; font-size: 14px; color: #333;">
      {por_cada_punto:}
      <li style="margin-bottom: 4px;">{punto_resumen}</li>
    </ul>
  </div>

  <!-- TEMAS TRATADOS -->
  <div style="margin-bottom: 20px;">
    <h3 style="font-size: 14px; color: #6B68FA; margin: 0 0 8px 0; text-transform: uppercase; letter-spacing: 0.5px;">Temas Tratados</h3>
    {por_cada_tema:}
    <div style="margin-bottom: 12px;">
      <p style="font-weight: 600; margin: 0 0 4px 0; color: #1C265D; font-size: 14px;">{numero}. {titulo_tema}</p>
      <p style="margin: 0; padding-left: 16px; font-size: 14px; color: #333;">{descripcion_tema}</p>
    </div>
  </div>

  <!-- DECISIONES -->
  {si_hay_decisiones:}
  <div style="margin-bottom: 20px;">
    <h3 style="font-size: 14px; color: #6B68FA; margin: 0 0 8px 0; text-transform: uppercase; letter-spacing: 0.5px;">Decisiones</h3>
    <ol style="margin: 0; padding-left: 20px; font-size: 14px; color: #333;">
      {por_cada_decision:}
      <li style="margin-bottom: 6px;">
        <strong>{decision}</strong>
        <br><span style="color: #666; font-size: 13px;">Decidido por: {quien}</span>
      </li>
    </ol>
  </div>

  <!-- ACCIONES PENDIENTES -->
  {si_hay_acciones:}
  <div style="margin-bottom: 20px;">
    <h3 style="font-size: 14px; color: #6B68FA; margin: 0 0 8px 0; text-transform: uppercase; letter-spacing: 0.5px;">Acciones Pendientes</h3>
    <table style="border-collapse: collapse; width: 100%; font-size: 14px;">
      <thead>
        <tr style="background: #f8f9fa;">
          <th style="padding: 6px 12px; border: 1px solid #dee2e6; text-align: left; font-weight: 600; width: 40px;">#</th>
          <th style="padding: 6px 12px; border: 1px solid #dee2e6; text-align: left; font-weight: 600;">Accion</th>
          <th style="padding: 6px 12px; border: 1px solid #dee2e6; text-align: left; font-weight: 600;">Responsable</th>
          <th style="padding: 6px 12px; border: 1px solid #dee2e6; text-align: left; font-weight: 600;">Fecha limite</th>
        </tr>
      </thead>
      <tbody>
        {por_cada_accion:}
        <tr>
          <td style="padding: 6px 12px; border: 1px solid #dee2e6; text-align: center;">{numero}</td>
          <td style="padding: 6px 12px; border: 1px solid #dee2e6;">{descripcion_accion}</td>
          <td style="padding: 6px 12px; border: 1px solid #dee2e6;">{responsable}</td>
          <td style="padding: 6px 12px; border: 1px solid #dee2e6;">{fecha_limite}</td>
        </tr>
      </tbody>
    </table>
  </div>

  <!-- PREGUNTAS ABIERTAS -->
  {si_hay_preguntas:}
  <div style="margin-bottom: 20px;">
    <h3 style="font-size: 14px; color: #6B68FA; margin: 0 0 8px 0; text-transform: uppercase; letter-spacing: 0.5px;">Preguntas Abiertas</h3>
    <ul style="margin: 0; padding-left: 20px; font-size: 14px; color: #333;">
      {por_cada_pregunta:}
      <li style="margin-bottom: 4px;">{pregunta}</li>
    </ul>
  </div>

  <!-- CAMBIOS DE ALCANCE DETECTADOS -->
  {si_hay_cambios_alcance:}
  <div style="margin-bottom: 20px; padding: 16px; background: #FFF8E1; border-left: 4px solid #E5B92B; border-radius: 4px;">
    <h3 style="font-size: 14px; color: #1C265D; margin: 0 0 8px 0; text-transform: uppercase; letter-spacing: 0.5px;">Cambios de Alcance Detectados</h3>
    <p style="font-size: 13px; color: #666; margin: 0 0 8px 0;">Los siguientes elementos no estan contemplados en el alcance actual y requieren presupuesto separado.</p>
    <ul style="margin: 0; padding-left: 20px; font-size: 14px; color: #333;">
      {por_cada_cambio:}
      <li style="margin-bottom: 6px;">
        <strong>{descripcion_cambio}</strong>
        <br><span style="color: #666; font-size: 13px;">Impacto: {impacto}</span>
      </li>
    </ul>
  </div>

  <!-- RIESGOS -->
  {si_hay_riesgos:}
  <div style="margin-bottom: 20px;">
    <h3 style="font-size: 14px; color: #6B68FA; margin: 0 0 8px 0; text-transform: uppercase; letter-spacing: 0.5px;">Riesgos Identificados</h3>
    <ul style="margin: 0; padding-left: 20px; font-size: 14px; color: #333;">
      {por_cada_riesgo:}
      <li style="margin-bottom: 4px;">{riesgo}</li>
    </ul>
  </div>

  <!-- PROXIMA REUNION -->
  {si_hay_proxima_reunion:}
  <div style="margin-bottom: 12px;">
    <h3 style="font-size: 14px; color: #6B68FA; margin: 0 0 8px 0; text-transform: uppercase; letter-spacing: 0.5px;">Proxima Reunion</h3>
    <p style="font-size: 14px; color: #333; margin: 0;">{fecha_proxima} - {temas_proxima}</p>
  </div>

  <!-- FOOTER -->
  <p style="font-size: 11px; color: #bbb; margin: 16px 0 0 0; text-align: right;">
    Generado el {DD/MM/YYYY HH:MM} | project-meeting-minutes v1.0
  </p>

</div>
```

---

## Notas para el LLM

### Secciones condicionales

Las secciones marcadas con `{si_hay_*:}` solo se incluyen si hay contenido. No generar secciones vacias.

Secciones obligatorias (siempre presentes):
- Header (badge + fecha)
- Asistentes
- Resumen ejecutivo
- Temas tratados

Secciones condicionales:
- Decisiones (solo si se tomaron decisiones)
- Acciones pendientes (solo si hay acciones)
- Preguntas abiertas (solo si quedaron preguntas sin resolver)
- Cambios de alcance (solo si se detectaron)
- Riesgos (solo si se identificaron)
- Proxima reunion (solo si se menciono)

### Colores de badge por tipo

| Tipo | badge_bg | badge_text |
|------|----------|------------|
| seguimiento | #6B68FA | #FFFFFF |
| kick-off | #E5B92B | #1C265D |
| validacion | #28a745 | #FFFFFF |
| formacion | #8AC2DB | #1C265D |
| analisis | #1C265D | #FFFFFF |

### Placeholders

Reemplazar todos los `{placeholders}` con datos reales. Los placeholders con `{por_cada_*:}` indican un loop que genera multiples elementos. Los placeholders con `{si_*:}` indican bloques condicionales.

### Formato de fechas

- En el header y body: DD/MM/YYYY (formato humano)
- En data-acta-fecha: YYYY-MM-DD (formato maquina, para busquedas)
- Fechas limite de acciones: DD/MM/YYYY

### Responsables

Indicar siempre nombre completo + empresa entre parentesis si es necesario distinguir:
- "Juan Garcia (Indaws)"
- "Maria Lopez (Hotel Portomar)"
