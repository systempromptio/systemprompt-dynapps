# Catálogo de Skills

## Qué es

La referencia completa de todos los skills del ecosistema Indaws organizados por fase del proyecto. Cada skill incluye: nombre, descripción, entrada, salida, repositorio y prerequisitos.

## Cuándo aplica

- Cuando el consultor pregunta "¿qué skill uso para...?"
- Para entender el flujo completo de un proyecto
- Para identificar dependencias entre skills

## Qué debe saber el consultor

### Fase 1: Pre-venta (repo: ai-claude-sales)

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| lead-data-collection | Pre-rellena formulario de descubrimiento desde CRM + web del cliente | crm.lead + web | Formulario pre-rellenado |
| demo-script-generator | Genera guion de demo personalizado + agenda para el cliente | crm.lead | Guion + Agenda HTML |
| demo-builder | Prepara instancia Odoo con datos realistas del sector | Oportunidad | Odoo demo listo |
| dev-assessment | Evalúa solicitudes de desarrollo: verifica Odoo estándar, Core, Studio antes de cotizar | Requisitos | Evaluación desarrollos |
| quotation-builder | Crea sale.order con apps, fases, horas, condiciones | Datos lead + dev-assessment | sale.order en Odoo |
| crm-health-monitor | Monitoriza pipeline CRM (paralelo durante toda la fase) | Pipeline CRM | Informe + Score |

### Fase 2: Análisis (repo: ai-claude-consultant)

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| kick-off-presentation | Presentación HTML + guion + Knowledge article para kick-off | sale.order + equipo | HTML + Knowledge |
| questionnaire-generator | Crea cuestionarios en Google Drive por cada app contratada | Apps (sale.order) | Google Docs en Drive |
| project-analysis | Genera análisis funcional completo con GAPs, soluciones y checklists | Cuestionarios rellenados | Análisis funcional .md |
| project-plan | Calcula horas, distribuye tareas en el tiempo y genera Gantt | Análisis funcional | Plan + Gantt |
| analysis-presentation *(futuro)* | Presentación profesional del análisis para el cliente | Análisis + Plan | Presentación HTML |

### Fase 3: Pre-proyecto (repo: ai-claude-consultant)

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| project-odoo-creator | Crea project.project + project.task en Odoo desde el plan | Plan de proyecto | Proyecto Odoo |
| client-request-manager (GENERAR) | Analiza tareas y crea peticiones al cliente | Tarea + Análisis | x_client_request draft |
| project-validation-drafting (PLANIFICAR) | Identifica validaciones necesarias, crea requests [VALIDACION] | Análisis + Tareas | Requests [VALIDACION] draft |
| training-planner (PLANIFICAR) | Genera calendario de formaciones del proyecto | Tareas + Análisis | Calendario + Checklists |

### Fase 4: Ejecución — 7 subflujos paralelos + monitorización

#### 4.1 Configuraciones / Odoo Studio

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| odoo-configurator | Ejecuta y documenta items de checklist (CONFIG + STUDIO/AUTO) | Tarea + Checklist | Odoo configurado |
| client-request-manager | Solicita datos faltantes al cliente | Datos faltantes | Request / Recordatorio |

#### 4.2 Formación

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| training-planner (CONTENIDO) | Genera materiales didácticos, guías, cheat sheets y ejercicios | Tarea Odoo | Materiales formación |
| project-meeting-minutes | Analiza transcript de reunión, coteja con tareas y checklists | Transcript | Acciones propuestas |

#### 4.3 Desarrollos a medida

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| dev-assessment | Valora el desarrollo y prepara documentación técnica | Requisitos | Doc técnica |
| technical-dev-validation *(futuro)* | Verifica que el desarrollo pasa todos los tests | Módulo dev | Resultado tests |
| client-request-manager | Envía validación del desarrollo al cliente | Doc validación | Email cliente |

#### 4.4 Migración de datos

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| importation-data-assistant | Analiza, normaliza e importa datos del cliente | Fichero datos | Datos importados + Doc |
| client-request-manager | Solicita info adicional o validación de datos importados | Info / Doc validación | Request / Email |

#### 4.5 Validaciones

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| project-validation-drafting (DOCUMENTAR) | Genera documento de validación en lenguaje de negocio | Tarea + Checklists | Doc validación |
| odoo-flow-tester | Testea flujos de negocio configurados como evidencia | Análisis + Odoo | Resultados test |

#### 4.6 Soporte durante implantación

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| service-desk | Analiza incidencias, diagnostica causa raíz, propone solución | Ticket | Resolución / Escalado |

#### 4.7 Pre-producción

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| go-live-validation *(futuro)* | Valida si el Go-Live es posible | Proyecto completo | Informe Go-Live |

#### 4.M Monitorización continua

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| consultant-daily-planner | Agenda diaria priorizada del consultor | Todos los proyectos | Agenda priorizada |
| project-health-monitor | Health Score (0-100) con métricas de timeline, checklists y horas | Proyecto Odoo | Informe salud |
| project-weekly-update | Actualización semanal HTML para el cliente | Health Monitor | project.update |
| project-health-fixer | Corrige problemas de gobernanza y propone plan de acción | Health Monitor | Correcciones Odoo |
| client-request-manager (FOLLOW-UP) | Gestión de emails, recordatorios y estado de requests | x_client_request | Emails + Estado |

### Fase 5: Post-proyecto (repo: ai-claude-consultant)

| Skill | Descripción | Entrada | Salida |
|-------|-------------|---------|--------|
| service-desk | Diagnóstico y solución de incidencias post go-live | Ticket cliente | Diagnóstico + Solución |
| service-desk-analytics *(futuro)* | Dashboard de soporte con acciones proactivas | Todos los tickets | Dashboard + Acciones |

### Transversal (usados en varias fases)

| Skill | Descripción | Entrada | Salida | Repo |
|-------|-------------|---------|--------|------|
| odoo-module-documenter | Documentación funcional de módulos (técnica o usuario final) | Código fuente | Documentación .md | consultant |
| odoo-screenshot-generator | Capturas automáticas con Playwright | Documento + Odoo | Screenshots PNG | consultant |
| odoo-test-data-generator | Datos de prueba realistas en CSV | Sector / Módulo | CSVs realistas | consultant |
| report-docx-template | Plantillas DOCX desde mockups | Mockup imagen | DOCX + Config XML | consultant |
| indaws-core-documenter | Documenta paquetes Indaws Core (ES + EN) | Código paquete | Producto Odoo + Knowledge | consultant |
| indaws-odoo-spreadsheets | Dashboards Odoo 18/19 via API | Modelo + visualizaciones | Dashboard JSON | consultant |

## Qué debe hacer el consultor

1. Identificar en qué fase del proyecto se encuentra
2. Consultar los skills disponibles para esa fase
3. Verificar prerequisitos antes de invocar un skill
4. Seguir el orden natural del flujo (no saltar fases)

## Skills relacionados

Todos — este reference es el índice central del ecosistema.

## Errores comunes

1. **Usar un skill de fase 4 sin haber completado fase 2-3**: El análisis y plan son prerequisitos
2. **No verificar prerequisitos**: Muchos skills dependen de la salida de otro (ej: weekly-update requiere health-monitor)
3. **Confundir modos del mismo skill**: client-request-manager tiene 3 modos (GENERAR/FOLLOW-UP/CERRAR), training-planner tiene 2 (PLANIFICAR/CONTENIDO)
4. **Olvidar los skills transversales**: odoo-flow-tester es útil en validaciones y en post-proyecto
5. **No actualizar el catálogo**: Los skills futuros se van creando — consulta los SKILL.md reales para el estado actual
