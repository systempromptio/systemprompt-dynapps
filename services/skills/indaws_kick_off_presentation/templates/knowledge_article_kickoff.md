# Template HTML para Knowledge Article de Kick-Off

## Instrucciones de Uso

Este template define la estructura HTML para el campo `body` de `knowledge.article`.
Usa estilos inline obligatorios (Odoo editor no soporta CSS externo).
Es un **documento de referencia**, no una presentacion. Pensado para leerse, no para proyectar.

---

## Template HTML

```html
<!-- CABECERA -->
<h1 style="color: #6B68FA; margin-bottom: 0.5rem;">Kick-Off — {nombre_cliente}</h1>
<p style="color: #8AC2DB; font-size: 14px; margin-bottom: 0;">
  Implantacion de Odoo {version_odoo} | Fecha: {fecha_kickoff}
</p>
<p style="color: #999; font-size: 12px;">
  Referencia: {sale_order_name} | Proyecto: {project_name}
</p>

<div style="margin: 30px 0; border-top: 3px solid #6B68FA;"></div>

<!-- INDICE -->
<div style="background-color: #f8f8f8; border: 1px solid #ddd; border-radius: 6px; padding: 16px 24px; margin: 16px 0;">
  <h3 style="margin-top: 0; color: #1C265D;">Contenido</h3>
  <ol style="color: #1C265D; line-height: 2;">
    <li>Equipo del proyecto</li>
    <li>Vision y alcance</li>
    <li>Metodologia de trabajo</li>
    <li>Compromisos y fechas</li>
    <li>Solicitudes pendientes</li>
    <li>Proximos pasos</li>
  </ol>
</div>

<div style="margin: 40px 0;"></div>

<!-- ============================================ -->
<!-- SECCION 1: EQUIPO DEL PROYECTO -->
<!-- ============================================ -->

<h2 style="color: #6B68FA; border-bottom: 2px solid #6B68FA; padding-bottom: 8px;">
  1. Equipo del proyecto
</h2>

<h3 style="color: #1C265D; margin-top: 20px;">Equipo Indaws</h3>

<table style="width: 100%; border-collapse: collapse; margin: 12px 0;">
  <thead>
    <tr style="border-bottom: 2px solid #6B68FA;">
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Nombre</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Rol en el proyecto</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Cargo</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Email</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Telefono</th>
    </tr>
  </thead>
  <tbody>
    {team_indaws_rows}
    <!-- Ejemplo:
    <tr style="border-bottom: 1px solid #eee;">
      <td style="padding: 8px 12px;"><b>{name}</b></td>
      <td style="padding: 8px 12px;">{rol_en_proyecto}</td>
      <td style="padding: 8px 12px;">{job_title}</td>
      <td style="padding: 8px 12px;">{work_email}</td>
      <td style="padding: 8px 12px;">{work_phone}</td>
    </tr>
    -->
  </tbody>
</table>

<p style="font-size: 13px; color: #666;">
  <b>Punto de contacto principal:</b> {pm_name} ({pm_email})
</p>

<h3 style="color: #1C265D; margin-top: 20px;">Equipo {nombre_cliente} (por definir)</h3>

<table style="width: 100%; border-collapse: collapse; margin: 12px 0;">
  <thead>
    <tr style="border-bottom: 2px solid #E5B92B;">
      <th style="text-align: left; padding: 8px 12px; color: #E5B92B;">Rol</th>
      <th style="text-align: left; padding: 8px 12px; color: #E5B92B;">Nombre</th>
      <th style="text-align: left; padding: 8px 12px; color: #E5B92B;">Area</th>
      <th style="text-align: left; padding: 8px 12px; color: #E5B92B;">Email</th>
    </tr>
  </thead>
  <tbody>
    {team_client_rows}
    <!-- Ejemplo:
    <tr style="border-bottom: 1px solid #eee;">
      <td style="padding: 8px 12px;"><b>SPOC</b></td>
      <td style="padding: 8px 12px;"><i>(por confirmar)</i></td>
      <td style="padding: 8px 12px;">-</td>
      <td style="padding: 8px 12px;">-</td>
    </tr>
    -->
  </tbody>
</table>

<div style="background: #FFF8E1; border-left: 4px solid #E5B92B; padding: 12px 16px; margin: 16px 0; border-radius: 0 4px 4px 0;">
  <b>Accion requerida:</b> {nombre_cliente} debe confirmar SPOC y Key Users en un maximo de 5 dias laborables.
</div>

<div style="margin: 40px 0; border-top: 1px solid #ddd;"></div>

<!-- ============================================ -->
<!-- SECCION 2: VISION Y ALCANCE -->
<!-- ============================================ -->

<h2 style="color: #6B68FA; border-bottom: 2px solid #6B68FA; padding-bottom: 8px;">
  2. Vision y alcance
</h2>

<h3 style="color: #1C265D; margin-top: 20px;">Vision del proyecto</h3>

{vision_narrative}
<!-- Parrafos con el resumen ejecutivo: situacion actual, que cambia, beneficios -->

<h3 style="color: #1C265D; margin-top: 20px;">Aplicaciones Odoo incluidas</h3>

<table style="width: 100%; border-collapse: collapse; margin: 12px 0;">
  <thead>
    <tr style="border-bottom: 2px solid #6B68FA;">
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Aplicacion</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Descripcion</th>
    </tr>
  </thead>
  <tbody>
    {apps_table_rows}
  </tbody>
</table>

<h3 style="color: #1C265D; margin-top: 20px;">Modulos Indaws Core</h3>

{core_modules_list}
<!-- Si no hay modulos Core, omitir esta subseccion -->

<h3 style="color: #1C265D; margin-top: 20px;">Alcance del proyecto</h3>

<div style="display: flex; gap: 20px; flex-wrap: wrap;">
  <div style="flex: 1; min-width: 280px; background: #f0faf0; border: 1px solid #c3e6cb; border-radius: 6px; padding: 16px;">
    <h4 style="color: #27ae60; margin-top: 0;">Incluido</h4>
    <ul style="color: #333; line-height: 1.8;">
      {included_items}
    </ul>
  </div>
  <div style="flex: 1; min-width: 280px; background: #fdf0f0; border: 1px solid #f5c6cb; border-radius: 6px; padding: 16px;">
    <h4 style="color: #e74c3c; margin-top: 0;">No incluido</h4>
    <ul style="color: #333; line-height: 1.8;">
      {excluded_items}
    </ul>
  </div>
</div>

<p style="font-size: 12px; color: #999; margin-top: 8px;">
  Cualquier funcionalidad fuera de este alcance sera evaluada y presupuestada por separado.
</p>

<div style="margin: 40px 0; border-top: 1px solid #ddd;"></div>

<!-- ============================================ -->
<!-- SECCION 3: METODOLOGIA -->
<!-- ============================================ -->

<h2 style="color: #6B68FA; border-bottom: 2px solid #6B68FA; padding-bottom: 8px;">
  3. Metodologia de trabajo
</h2>

<h3 style="color: #1C265D; margin-top: 20px;">Fases del proyecto</h3>

<table style="width: 100%; border-collapse: collapse; margin: 12px 0;">
  <thead>
    <tr style="border-bottom: 2px solid #6B68FA;">
      <th style="text-align: center; padding: 8px 12px; color: #6B68FA; width: 40px;">#</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Fase</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Descripcion</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Entregable</th>
    </tr>
  </thead>
  <tbody>
    <tr style="border-bottom: 1px solid #eee; background: #FFF8E1;">
      <td style="padding: 8px 12px; text-align: center; font-weight: bold;">1</td>
      <td style="padding: 8px 12px;"><b>Kick-Off</b></td>
      <td style="padding: 8px 12px;">Alineacion, equipo, compromisos</td>
      <td style="padding: 8px 12px;">Acta de kick-off</td>
    </tr>
    <tr style="border-bottom: 1px solid #eee;">
      <td style="padding: 8px 12px; text-align: center; font-weight: bold;">2</td>
      <td style="padding: 8px 12px;"><b>Analisis</b></td>
      <td style="padding: 8px 12px;">Sesiones con Key Users, diseno de solucion</td>
      <td style="padding: 8px 12px;">Documento de analisis funcional</td>
    </tr>
    <tr style="border-bottom: 1px solid #eee;">
      <td style="padding: 8px 12px; text-align: center; font-weight: bold;">3</td>
      <td style="padding: 8px 12px;"><b>Implantacion</b></td>
      <td style="padding: 8px 12px;">Configuracion, desarrollo, migracion</td>
      <td style="padding: 8px 12px;">Entorno Odoo configurado</td>
    </tr>
    <tr style="border-bottom: 1px solid #eee;">
      <td style="padding: 8px 12px; text-align: center; font-weight: bold;">4</td>
      <td style="padding: 8px 12px;"><b>UAT</b></td>
      <td style="padding: 8px 12px;">Testing con el cliente</td>
      <td style="padding: 8px 12px;">Informe de UAT aprobado</td>
    </tr>
    <tr style="border-bottom: 1px solid #eee;">
      <td style="padding: 8px 12px; text-align: center; font-weight: bold;">5</td>
      <td style="padding: 8px 12px;"><b>Go-Live</b></td>
      <td style="padding: 8px 12px;">Salida a produccion</td>
      <td style="padding: 8px 12px;">Sistema en produccion</td>
    </tr>
    <tr style="border-bottom: 1px solid #eee;">
      <td style="padding: 8px 12px; text-align: center; font-weight: bold;">6</td>
      <td style="padding: 8px 12px;"><b>Soporte Post-Arranque</b></td>
      <td style="padding: 8px 12px;">Acompanamiento primeras semanas</td>
      <td style="padding: 8px 12px;">Cierre de proyecto</td>
    </tr>
  </tbody>
</table>

<h3 style="color: #1C265D; margin-top: 20px;">El Analisis en detalle</h3>

{analysis_detail}
<!-- Subfases del analisis con lo que esperamos del cliente -->

<h3 style="color: #1C265D; margin-top: 20px;">Plan de comunicacion</h3>

<table style="width: 100%; border-collapse: collapse; margin: 12px 0;">
  <thead>
    <tr style="border-bottom: 2px solid #6B68FA;">
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Canal</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Frecuencia</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Participantes</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Objetivo</th>
    </tr>
  </thead>
  <tbody>
    {communication_rows}
  </tbody>
</table>

<h3 style="color: #1C265D; margin-top: 20px;">Gestion de cambios</h3>

<p style="line-height: 1.8;">
  Las solicitudes fuera de alcance siguen este proceso:
</p>
<ol style="line-height: 2; color: #333;">
  <li><b>Solicitud:</b> El cliente identifica una necesidad no contemplada</li>
  <li><b>Evaluacion:</b> Indaws analiza el impacto (horas y timeline)</li>
  <li><b>Propuesta:</b> Presupuesto adicional o reajuste de prioridades</li>
  <li><b>Aprobacion:</b> El cliente decide si proceder</li>
</ol>

<div style="margin: 40px 0; border-top: 1px solid #ddd;"></div>

<!-- ============================================ -->
<!-- SECCION 4: COMPROMISOS Y FECHAS -->
<!-- ============================================ -->

<h2 style="color: #6B68FA; border-bottom: 2px solid #6B68FA; padding-bottom: 8px;">
  4. Compromisos y fechas
</h2>

<h3 style="color: #1C265D; margin-top: 20px;">Timeline del proyecto</h3>

{timeline_table}
<!-- Tabla con las fases, fechas inicio/fin, hitos clave -->

<h3 style="color: #1C265D; margin-top: 20px;">Calendario de sesiones de analisis</h3>

<table style="width: 100%; border-collapse: collapse; margin: 12px 0;">
  <thead>
    <tr style="border-bottom: 2px solid #6B68FA;">
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Semana</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Dia</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Hora</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Sesion</th>
      <th style="text-align: left; padding: 8px 12px; color: #6B68FA;">Key Users</th>
    </tr>
  </thead>
  <tbody>
    {analysis_sessions_rows}
  </tbody>
</table>

<h3 style="color: #1C265D; margin-top: 20px;">Compromisos mutuos</h3>

<div style="display: flex; gap: 20px; flex-wrap: wrap;">
  <div style="flex: 1; min-width: 280px; background: #f0f0ff; border: 1px solid #c5c4fe; border-radius: 6px; padding: 16px;">
    <h4 style="color: #6B68FA; margin-top: 0;">Indaws se compromete a</h4>
    <ul style="color: #333; line-height: 1.8;">
      {indaws_commitments}
    </ul>
  </div>
  <div style="flex: 1; min-width: 280px; background: #FFF8E1; border: 1px solid #FFE082; border-radius: 6px; padding: 16px;">
    <h4 style="color: #E5B92B; margin-top: 0;">{nombre_cliente} se compromete a</h4>
    <ul style="color: #333; line-height: 1.8;">
      {client_commitments}
    </ul>
  </div>
</div>

<h3 style="color: #1C265D; margin-top: 20px;">Impacto de retrasos</h3>

<div style="background: #fff5f5; border: 1px solid #f5c6cb; border-radius: 6px; padding: 16px; margin: 12px 0;">
  {impact_narrative}
  <!-- Cadena causa-efecto en formato texto/lista -->
</div>

<p style="font-size: 13px; color: #666; font-style: italic;">
  Los plazos del proyecto dependen de la velocidad de respuesta de ambas partes.
</p>

<div style="margin: 40px 0; border-top: 1px solid #ddd;"></div>

<!-- ============================================ -->
<!-- SECCION 5: SOLICITUDES PENDIENTES -->
<!-- ============================================ -->

<h2 style="color: #6B68FA; border-bottom: 2px solid #6B68FA; padding-bottom: 8px;">
  5. Solicitudes pendientes
</h2>

<table style="width: 100%; border-collapse: collapse; margin: 12px 0;">
  <thead>
    <tr style="border-bottom: 2px solid #E5B92B;">
      <th style="text-align: center; padding: 8px 12px; color: #E5B92B;">Prioridad</th>
      <th style="text-align: left; padding: 8px 12px; color: #E5B92B;">Solicitud</th>
      <th style="text-align: left; padding: 8px 12px; color: #E5B92B;">Responsable</th>
      <th style="text-align: center; padding: 8px 12px; color: #E5B92B;">Fecha limite</th>
    </tr>
  </thead>
  <tbody>
    {requests_table_rows}
    <!-- Ejemplo:
    <tr style="border-bottom: 1px solid #eee;">
      <td style="padding: 8px 12px; text-align: center;">
        <span style="background: #e74c3c; color: #fff; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: bold;">CRITICA</span>
      </td>
      <td style="padding: 8px 12px;">Confirmar SPOC del proyecto</td>
      <td style="padding: 8px 12px;">Direccion {nombre_cliente}</td>
      <td style="padding: 8px 12px; text-align: center;">{fecha_limite}</td>
    </tr>
    -->
  </tbody>
</table>

<div style="margin: 40px 0; border-top: 1px solid #ddd;"></div>

<!-- ============================================ -->
<!-- SECCION 6: PROXIMOS PASOS -->
<!-- ============================================ -->

<h2 style="color: #6B68FA; border-bottom: 2px solid #6B68FA; padding-bottom: 8px;">
  6. Proximos pasos
</h2>

<ol style="line-height: 2.2; color: #333; font-size: 14px;">
  {next_steps_items}
  <!-- Ejemplo:
  <li><b>Indaws envia acta del kick-off</b> — {pm_name}, {fecha +1 dia}</li>
  <li><b>Cliente confirma SPOC y Key Users</b> — {nombre_cliente}, {fecha +3 dias}</li>
  <li><b>Primera sesion de analisis</b> — {fecha primera sesion}</li>
  -->
</ol>

<div style="background: #f0f0ff; border-left: 4px solid #6B68FA; padding: 12px 16px; margin: 20px 0; border-radius: 0 4px 4px 0;">
  <b>A partir de hoy, el proyecto esta en marcha.</b>
</div>

<!-- FOOTER -->
<div style="margin-top: 40px; padding-top: 16px; border-top: 3px solid #6B68FA; text-align: center;">
  <p style="font-size: 12px; color: #999;">
    Documento generado automaticamente — <b>Indaws Business Solutions S.L.</b> — Odoo Gold Partner
  </p>
</div>
```

---

## Placeholders

Todos los valores entre `{...}` deben ser reemplazados con datos reales. Ver la lista completa en el template de la presentacion (`presentacion_kickoff.md`).

### Placeholders especificos del Knowledge Article

- `{sale_order_name}` — Referencia del sale.order (ej: "PR3492")
- `{project_name}` — Nombre del proyecto en Odoo
- `{team_indaws_rows}` — Filas HTML de la tabla del equipo Indaws
- `{team_client_rows}` — Filas HTML de la tabla del equipo del cliente (por definir)
- `{vision_narrative}` — Parrafos HTML con el resumen ejecutivo
- `{apps_table_rows}` — Filas de la tabla de aplicaciones
- `{core_modules_list}` — Lista HTML de modulos Core (o vacio si no hay)
- `{analysis_detail}` — Detalle de las subfases del analisis
- `{communication_rows}` — Filas de la tabla de comunicacion
- `{timeline_table}` — Tabla con fases y fechas
- `{analysis_sessions_rows}` — Filas del calendario de analisis
- `{requests_table_rows}` — Filas de solicitudes con prioridad
- `{next_steps_items}` — Items `<li>` de proximos pasos
- `{impact_narrative}` — Cadena causa-efecto en HTML

### Reglas de estilo
- Headers h2: `color: #6B68FA; border-bottom: 2px solid #6B68FA;`
- Headers h3: `color: #1C265D;`
- Tablas: Headers con `color: #6B68FA`, filas alternas con `#F5F7FA`
- Callouts informativos: `background: #f0f0ff; border-left: 4px solid #6B68FA;`
- Callouts de accion: `background: #FFF8E1; border-left: 4px solid #E5B92B;`
- Callouts de alerta: `background: #fff5f5; border: 1px solid #f5c6cb;`
- Badges de prioridad: CRITICA=#e74c3c, ALTA=#E5B92B, MEDIA=#8AC2DB
