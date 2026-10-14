# Template: Plan de Formacion en Knowledge

Este template genera HTML con estilos inline para articulos `knowledge.article` en Odoo 19.

**IMPORTANTE**: Odoo 19 no procesa clases CSS ni Bootstrap. Todo debe ir con estilos inline.

## Template HTML

```html
<div style="font-family: 'Montserrat', sans-serif; color: #1C265D; max-width: 900px;">

  <h1 style="font-family: 'Montserrat', sans-serif; color: #1C265D; font-size: 28px; font-weight: 700; margin-bottom: 8px;">
    Plan de Formacion — {NOMBRE_PROYECTO}
  </h1>

  <p style="font-family: 'Montserrat', sans-serif; color: #555; font-size: 14px; margin-bottom: 24px;">
    <strong>Formato:</strong> {FORMATO} &nbsp;|&nbsp;
    <strong>Duracion por sesion:</strong> {DURACION_SESION} &nbsp;|&nbsp;
    <strong>Frecuencia:</strong> {FRECUENCIA}
  </p>

  <h2 style="font-family: 'Montserrat', sans-serif; color: #1C265D; font-size: 20px; font-weight: 600; margin-top: 32px; margin-bottom: 16px; border-bottom: 2px solid #1C265D; padding-bottom: 6px;">
    Calendario de Sesiones
  </h2>

  <table style="width: 100%; border-collapse: collapse; font-family: 'Montserrat', sans-serif; font-size: 13px; margin-bottom: 32px;">
    <thead>
      <tr style="background-color: #1C265D; color: white;">
        <th style="padding: 10px 12px; text-align: left; border: 1px solid #1C265D; font-weight: 600;">Semana</th>
        <th style="padding: 10px 12px; text-align: left; border: 1px solid #1C265D; font-weight: 600;">Fecha</th>
        <th style="padding: 10px 12px; text-align: left; border: 1px solid #1C265D; font-weight: 600;">Sesion</th>
        <th style="padding: 10px 12px; text-align: center; border: 1px solid #1C265D; font-weight: 600;">Duracion</th>
      </tr>
    </thead>
    <tbody>
      <!-- Repetir por cada sesion -->
      <tr style="background-color: #ffffff;">
        <td style="padding: 8px 12px; border: 1px solid #ddd;">{SEMANA_N}</td>
        <td style="padding: 8px 12px; border: 1px solid #ddd;">{FECHA}</td>
        <td style="padding: 8px 12px; border: 1px solid #ddd;"><strong>{NOMBRE_SESION}</strong></td>
        <td style="padding: 8px 12px; border: 1px solid #ddd; text-align: center;">{DURACION}</td>
      </tr>
      <!-- Fila alternada -->
      <tr style="background-color: #f8f9fa;">
        <td style="padding: 8px 12px; border: 1px solid #ddd;">{SEMANA_N}</td>
        <td style="padding: 8px 12px; border: 1px solid #ddd;">{FECHA}</td>
        <td style="padding: 8px 12px; border: 1px solid #ddd;"><strong>{NOMBRE_SESION}</strong></td>
        <td style="padding: 8px 12px; border: 1px solid #ddd; text-align: center;">{DURACION}</td>
      </tr>
      <!-- Las reuniones de seguimiento van en estilo mas discreto -->
      <tr style="background-color: #ffffff;">
        <td style="padding: 8px 12px; border: 1px solid #ddd;">{SEMANA_N}</td>
        <td style="padding: 8px 12px; border: 1px solid #ddd;">{FECHA}</td>
        <td style="padding: 8px 12px; border: 1px solid #ddd; color: #888; font-style: italic;">Reunion de seguimiento</td>
        <td style="padding: 8px 12px; border: 1px solid #ddd; text-align: center; color: #888;">30min</td>
      </tr>
    </tbody>
  </table>

  <h2 style="font-family: 'Montserrat', sans-serif; color: #1C265D; font-size: 20px; font-weight: 600; margin-top: 32px; margin-bottom: 16px; border-bottom: 2px solid #1C265D; padding-bottom: 6px;">
    Detalle de las Formaciones
  </h2>

  <!-- Repetir por cada formacion (NO reuniones de seguimiento) -->
  <div style="margin-bottom: 24px; padding: 16px; border-left: 4px solid #1C265D; background-color: #f8f9fa;">
    <h3 style="font-family: 'Montserrat', sans-serif; color: #1C265D; font-size: 16px; font-weight: 600; margin: 0 0 8px 0;">
      {NOMBRE_SESION}
    </h3>
    <p style="font-family: 'Montserrat', sans-serif; color: #555; font-size: 13px; margin: 0 0 4px 0;">
      <strong>Fecha:</strong> {FECHA} &nbsp;|&nbsp; <strong>Duracion:</strong> {DURACION}
    </p>
    <p style="font-family: 'Montserrat', sans-serif; color: #333; font-size: 13px; margin: 8px 0 0 0;">
      {DESCRIPCION_SESION}
    </p>
  </div>

</div>
```

## Notas de uso

1. **Colores de fila alternados**: Usar `#ffffff` y `#f8f9fa` alternadamente
2. **Reuniones de seguimiento**: Mostrar en gris cursiva (`color: #888; font-style: italic`)
3. **Formaciones**: Mostrar en negrita con color normal
4. **NO usar clases CSS**: Todo inline
5. **Crear con `create`**: NUNCA usar multiples `write` al body — causa corrupcion del historial del editor
6. **Escapar comillas**: En el JSON de la API, escapar las comillas dobles del HTML
