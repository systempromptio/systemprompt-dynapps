# Template HTML para project.update

## Instrucciones de Uso

Este template contiene el HTML con estilos inline para el campo `description` de `project.update`.
Los placeholders `{...}` deben ser reemplazados con datos del JSON y narrativa del LLM.

**Soporte bilingue**: El template usa placeholders `{i18n_*}` para todas las
cadenas fijas. Sustituyelos segun el valor de `data.language` (`es` o `en`)
usando la tabla de traducciones de abajo. `create_update.py` aborta si el
HTML final mezcla idiomas.

**Placeholders de datos** (rellenar directamente desde JSON):
- `{project_name}` - Nombre del proyecto
- `{week_start_display}` - Fecha inicio semana (DD/MM/YYYY)
- `{week_end_display}` - Fecha fin semana (DD/MM/YYYY)
- `{C_pct}` - Porcentaje de progreso global (completado)
- `{T_pct}` - Porcentaje de tiempo transcurrido del proyecto
- `{progress_color}` - Color hex del progreso
- `{task_rows}` - Filas HTML de la tabla de tareas (incluye columnas Inicio y Fin)
- `{completed_items_section}` - Seccion HTML de "Trabajos terminados" (solo si hay checklists completados esta semana, vacio si no hay)
- `{project_dates_line}` - Linea HTML con fechas del proyecto (inicio, fin, y opcionalmente salida a produccion)

**Placeholders narrativos** (redactados por el LLM en el idioma solicitado):
- `{narrative_work_done}` - Parrafo narrativo del trabajo realizado (SIN horas, solo descripcion funcional)
- `{progress_narrative}` - Interpretacion cualitativa del progreso
- `{next_steps_items}` - Items `<li>` de proximos pasos

**IMPORTANTE**: Las horas son datos internos. NO incluir horas, tabla de horas, ni desglose de tiempo en el HTML del cliente. Los timesheets se usan SOLO como fuente para entender que trabajo se ha hecho y redactar la narrativa.

## Tabla de traducciones i18n

| Placeholder | es | en |
|---|---|---|
| `{i18n_header_title}` | Seguimiento Semanal | Weekly Update |
| `{i18n_week_label}` | Semana del {week_start_display} al {week_end_display} | Week of {week_start_display} to {week_end_display} |
| `{i18n_section_work_done}` | Trabajo Realizado | Work Done |
| `{i18n_section_progress}` | Progreso del Proyecto | Project Progress |
| `{i18n_section_tasks}` | Estado de Tareas | Task Status |
| `{i18n_section_next_steps}` | Proximos Pasos | Next Steps |
| `{i18n_col_task}` | Tarea | Task |
| `{i18n_col_stage}` | Etapa | Stage |
| `{i18n_col_start}` | Inicio | Start |
| `{i18n_col_end}` | Fin | End |
| `{i18n_col_progress}` | Progreso | Progress |
| `{i18n_progress_caption}` | Progreso del proyecto / Tiempo transcurrido | Project progress / Time elapsed |
| `{i18n_completed_label}` | Trabajos terminados: | Completed items: |
| `{i18n_footer}` | Actualizacion generada automaticamente | Automatically generated update |

**Nota de idioma**: Los marcadores que detecta `create_update.py` como
contaminacion son los de la columna contraria al idioma solicitado. Es decir,
si `language=es`, el HTML NO debe contener "Weekly Update", "Work Done",
"Next Steps", etc.; si `language=en`, NO debe contener "Seguimiento Semanal",
"Trabajo Realizado", "Proximos Pasos", etc.

---

## Template HTML

```html
<div style="font-family: Arial, Helvetica, sans-serif; color: #333333; max-width: 700px;">

  <!-- HEADER -->
  <div style="background-color: #875A7B; padding: 20px 25px; border-radius: 4px 4px 0 0;">
    <h2 style="color: #ffffff; margin: 0; font-size: 20px;">
      {i18n_header_title} - {project_name}
    </h2>
    <p style="color: #e8d5e0; margin: 5px 0 0 0; font-size: 14px;">
      {i18n_week_label}
    </p>
  </div>

  <div style="padding: 20px 25px; border: 1px solid #e0e0e0; border-top: none; border-radius: 0 0 4px 4px;">

    <!-- SECCION 1: TRABAJO REALIZADO -->
    <h3 style="color: #875A7B; font-size: 16px; border-bottom: 2px solid #875A7B; padding-bottom: 8px; margin-top: 0;">
      {i18n_section_work_done}
    </h3>

    <p style="font-size: 14px; line-height: 1.6; color: #555555;">
      {narrative_work_done}
    </p>

    {completed_items_section}

    <!-- SECCION 2: PROGRESO DEL PROYECTO -->
    <h3 style="color: #875A7B; font-size: 16px; border-bottom: 2px solid #875A7B; padding-bottom: 8px;">
      {i18n_section_progress}
    </h3>

    <div style="text-align: center; padding: 15px 0;">
      <div style="display: inline-block; background-color: {progress_color}; color: #ffffff; font-size: 28px; font-weight: bold; padding: 15px 30px; border-radius: 8px;">
        {C_pct}%
      </div>
      <div style="display: inline-block; background-color: #555555; color: #ffffff; font-size: 28px; font-weight: bold; padding: 15px 30px; border-radius: 8px; margin-left: 15px;">
        {T_pct}%
      </div>
      <p style="font-size: 12px; color: #888888; margin-top: 5px;">{i18n_progress_caption}</p>
      {project_dates_line}
    </div>

    <p style="font-size: 14px; line-height: 1.6; color: #555555;">
      {progress_narrative}
    </p>

    <!-- SECCION 3: ESTADO DE TAREAS -->
    <h3 style="color: #875A7B; font-size: 16px; border-bottom: 2px solid #875A7B; padding-bottom: 8px;">
      {i18n_section_tasks}
    </h3>

    <table style="width: 100%; border-collapse: collapse; margin: 15px 0; font-size: 13px;">
      <thead>
        <tr style="background-color: #f5f0f3;">
          <th style="text-align: left; padding: 8px 12px; border: 1px solid #e0e0e0; color: #875A7B;">{i18n_col_task}</th>
          <th style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0; color: #875A7B;">{i18n_col_stage}</th>
          <th style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0; color: #875A7B;">{i18n_col_start}</th>
          <th style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0; color: #875A7B;">{i18n_col_end}</th>
          <th style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0; color: #875A7B;">{i18n_col_progress}</th>
        </tr>
      </thead>
      <tbody>
        {task_rows}
      </tbody>
    </table>

    <!-- SECCION 4: PROXIMOS PASOS -->
    <h3 style="color: #875A7B; font-size: 16px; border-bottom: 2px solid #875A7B; padding-bottom: 8px;">
      {i18n_section_next_steps}
    </h3>

    <ul style="font-size: 14px; line-height: 1.8; color: #555555; padding-left: 20px;">
      {next_steps_items}
    </ul>

  </div>

  <!-- FOOTER -->
  <div style="text-align: center; padding: 15px; font-size: 11px; color: #999999;">
    {i18n_footer} - <strong>Indaws</strong>
  </div>

</div>
```

---

## Formato de Filas HTML

### task_rows (una por tarea)
```html
<tr>
  <td style="padding: 8px 12px; border: 1px solid #e0e0e0;">{task_name}</td>
  <td style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0;">{stage}</td>
  <td style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0; font-size: 12px;">{date_start}</td>
  <td style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0; font-size: 12px;">{date_end}</td>
  <td style="text-align: center; padding: 8px 12px; border: 1px solid #e0e0e0;">
    <span style="background-color: {task_color}; color: #ffffff; padding: 2px 8px; border-radius: 10px; font-size: 12px;">{C_pct}%</span>
  </td>
</tr>
```

### task_color por progreso
- 100% → `#27ae60` (verde)
- 50-99% → `#f39c12` (naranja)
- 0-49% → `#e74c3c` (rojo)

### completed_items_section (solo si hay checklists completados)
```html
<p style="font-size: 14px; font-weight: bold; color: #875A7B; margin-bottom: 5px;">{i18n_completed_label}</p>
<ul style="font-size: 13px; line-height: 1.6; color: #555555; padding-left: 20px; margin-top: 5px;">
  <li>{item_name} ({task_name})</li>
  ...
</ul>
```
Si `weekly_checklists.total_completed == 0`, no incluir esta seccion (dejar vacio).

### next_steps_items
```html
<li>{step_text}</li>
```
