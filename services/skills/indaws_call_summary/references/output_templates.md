# Templates: Call Summary Outputs

## Formato completo

```
=============================================
  RESUMEN DE LLAMADA
  {empresa} | {fecha} | {tipo}
=============================================

--- 1. RESUMEN EJECUTIVO ---
Duracion: {estimada o real}
Asistentes: {lista}
Resultado: {positivo/neutral/negativo}

- {punto_clave_1}
- {punto_clave_2}
- {punto_clave_3}
- {punto_clave_4}
- {punto_clave_5}

Senal: {senal_de_compra_o_riesgo_identificada}

--- 2. ACCIONES ---
| # | Accion | Responsable | Fecha limite |
|---|--------|-------------|--------------|
| 1 | {accion_concreta} | {quien} | {cuando} |
| 2 | {accion_concreta} | {quien} | {cuando} |
| 3 | {accion_concreta} | {quien} | {cuando} |

--- 3. EMAIL DE SEGUIMIENTO (borrador) ---
Asunto: {asunto_conciso}

Hola {nombre},

{Parrafo 1: Agradecimiento + referencia a la reunion}

{Parrafo 2: Resumen de lo acordado (2-3 puntos clave)}

{Parrafo 3: Proximos pasos con fechas concretas}

{Parrafo 4: Cierre + disponibilidad}

Un saludo,
{nombre_comercial}

--- 4. NOTA INTERNA (para Odoo chatter) ---
Llamada {tipo} con {empresa} ({fecha}):
- {punto_1}
- {punto_2}
- {punto_3}
Proximos pasos: {resumen_acciones}
Senal: {senal_compra_o_riesgo}

--- 5. ACTUALIZACION CRM SUGERIDA ---
Etapa actual:    {etapa_actual}
Etapa sugerida:  {etapa_nueva} (motivo: {razon})
Probabilidad:    {prob_actual}% -> {prob_nueva}% (motivo: {razon})
Revenue:         {rev_actual} EUR -> {rev_nuevo} EUR (si cambio)
Siguiente accion: {actividad_a_crear}
Fecha limite:    {fecha}
=============================================
```

El bloque de texto de arriba es el formato de **revision** que se muestra al usuario
en el chat. Lo que se **publica en el chatter de Odoo** es la version HTML de abajo.

## Publicacion en el chatter de Odoo (HTML)

El chatter de Odoo es HTML inline: NO carga CSS externo ni clases `iw-*`, asi que se
usa el sistema de diseno Indaws traducido a **estilos inline** (maxima compatibilidad
con el sanitizador de `mail.message`). Componentes y colores:

- Header band + `iw-summary-box` (resumen ejecutivo): fondo navy `#1C265D`, acento oro `#E5B92B`
- `iw-callout-warning` (senal): borde izq. oro `#E5B92B`, fondo `#fffbf0`, titulo `#c99e1a`
- `iw-callout` (email): borde izq. lila `#6B68FA`, fondo `#f7f8fc`
- `iw-table iw-table-striped` (acciones, CRM): cabecera navy `#1C265D`/blanco, filas pares `#f7f8fc`, bordes `#e2e8f0`
- Tipografia `'Montserrat', Verdana, sans-serif`, texto `#1C265D`

Reglas:
1. **Escapar** entidades del contenido (`&`->`&amp;`, `<`->`&lt;`, `>`->`&gt;`) antes de insertarlo.
2. Usar `<br>` para los saltos de linea del cuerpo del email (no `\n`).
3. Publicar con `subtype_xmlid="mail.mt_note"` y `message_type="comment"` (nota interna, NO email a contactos).
4. Omitir secciones vacias (p.ej. si no hay cambio de CRM, no renderizar esa tabla).

### Plantilla HTML (rellenar placeholders, pasar como `body`)

```html
<div style="font-family:'Montserrat',Verdana,sans-serif;color:#1C265D;font-size:14px;line-height:1.6;max-width:760px;">

  <!-- Header band -->
  <div style="background:#1C265D;border-radius:12px 12px 0 0;padding:18px 24px;">
    <div style="color:#E5B92B;font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em;margin:0 0 4px;">Resumen de llamada</div>
    <div style="color:#FFFFFF;font-size:18px;font-weight:700;">{empresa}</div>
    <div style="color:#8AC2DB;font-size:13px;margin-top:4px;">{fecha} &nbsp;&middot;&nbsp; {tipo} &nbsp;&middot;&nbsp; {resultado}</div>
  </div>

  <!-- Body -->
  <div style="background:#FFFFFF;border:1px solid #e2e8f0;border-top:none;border-radius:0 0 12px 12px;padding:24px;">

    <p style="margin:0 0 18px;color:#4a5568;font-size:13px;">
      <strong style="color:#1C265D;">Duracion:</strong> {duracion} &nbsp;|&nbsp;
      <strong style="color:#1C265D;">Asistentes:</strong> {asistentes}
    </p>

    <!-- 1. Resumen ejecutivo (summary-box) -->
    <div style="background:#1C265D;color:#FFFFFF;border-radius:12px;padding:18px 22px;margin:0 0 20px;">
      <div style="color:#E5B92B;font-size:12px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em;margin-bottom:10px;">Resumen ejecutivo</div>
      <ul style="margin:0;padding-left:18px;">
        <li style="margin-bottom:6px;">{punto_clave_1}</li>
        <li style="margin-bottom:6px;">{punto_clave_2}</li>
        <li style="margin-bottom:6px;">{punto_clave_3}</li>
      </ul>
    </div>

    <!-- Senal (callout-warning) -->
    <div style="background:#fffbf0;border-left:4px solid #E5B92B;border-radius:0 12px 12px 0;padding:14px 18px;margin:0 0 22px;">
      <div style="color:#c99e1a;font-weight:700;font-size:14px;margin-bottom:2px;">Senal</div>
      <div style="color:#1C265D;">{senal_de_compra_o_riesgo}</div>
    </div>

    <!-- 2. Acciones (table striped) -->
    <div style="font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:0.03em;color:#6B68FA;margin:0 0 8px;">Acciones</div>
    <table style="border-collapse:separate;border-spacing:0;width:100%;border:1px solid #e2e8f0;border-radius:8px;overflow:hidden;font-size:13px;margin:0 0 22px;">
      <thead>
        <tr>
          <th style="background:#1C265D;color:#FFFFFF;font-weight:600;text-transform:uppercase;letter-spacing:0.03em;font-size:11px;padding:10px 12px;text-align:left;">#</th>
          <th style="background:#1C265D;color:#FFFFFF;font-weight:600;text-transform:uppercase;letter-spacing:0.03em;font-size:11px;padding:10px 12px;text-align:left;">Accion</th>
          <th style="background:#1C265D;color:#FFFFFF;font-weight:600;text-transform:uppercase;letter-spacing:0.03em;font-size:11px;padding:10px 12px;text-align:left;">Responsable</th>
          <th style="background:#1C265D;color:#FFFFFF;font-weight:600;text-transform:uppercase;letter-spacing:0.03em;font-size:11px;padding:10px 12px;text-align:left;">Fecha limite</th>
        </tr>
      </thead>
      <tbody>
        <tr style="background:#FFFFFF;">
          <td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">1</td>
          <td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">{accion}</td>
          <td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">{responsable}</td>
          <td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">{fecha}</td>
        </tr>
        <tr style="background:#f7f8fc;">
          <td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">2</td>
          <td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">{accion}</td>
          <td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">{responsable}</td>
          <td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">{fecha}</td>
        </tr>
      </tbody>
    </table>

    <!-- 3. Email de seguimiento (callout) -->
    <div style="font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:0.03em;color:#6B68FA;margin:0 0 8px;">Email de seguimiento (borrador)</div>
    <div style="background:#f7f8fc;border-left:4px solid #6B68FA;border-radius:0 12px 12px 0;padding:14px 18px;margin:0 0 22px;">
      <div style="margin-bottom:8px;"><strong>Asunto:</strong> {asunto}</div>
      <div style="color:#1C265D;">{cuerpo_email_con_br}</div>
    </div>

    <!-- 5. Actualizacion CRM sugerida (table) -->
    <div style="font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:0.03em;color:#6B68FA;margin:0 0 8px;">Actualizacion CRM sugerida</div>
    <table style="border-collapse:separate;border-spacing:0;width:100%;border:1px solid #e2e8f0;border-radius:8px;overflow:hidden;font-size:13px;">
      <tbody>
        <tr style="background:#FFFFFF;"><td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;font-weight:600;width:38%;">Etapa</td><td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">{etapa_actual} &rarr; {etapa_sugerida}</td></tr>
        <tr style="background:#f7f8fc;"><td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;font-weight:600;">Probabilidad</td><td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">{prob_actual}% &rarr; {prob_nueva}%</td></tr>
        <tr style="background:#FFFFFF;"><td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;font-weight:600;">Revenue</td><td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">{rev_actual} &rarr; {rev_nuevo} EUR</td></tr>
        <tr style="background:#f7f8fc;"><td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;font-weight:600;">Siguiente accion</td><td style="padding:9px 12px;border-bottom:1px solid #e2e8f0;">{actividad} ({fecha_limite})</td></tr>
      </tbody>
    </table>

  </div>
</div>
```

## Reglas del Email de Seguimiento

- Texto plano (sin HTML, sin markdown, sin negritas)
- Espanol profesional
- Maximo 150 palabras
- Tono: profesional, cercano, orientado a accion
- Siempre incluir proximos pasos concretos con fechas
- No usar frases genericas ("espero que estes bien", "quedo a tu disposicion")
