# Ejemplo: Especificación de Capturas para indaws_crm_prospection

Este documento muestra cómo especificar qué capturas generar para un módulo Odoo.

Este es el formato que el skill `odoo-screenshot-generator` lee cuando se le pasa documentación de un módulo.

---

## Capturas Requeridas

### Captura 1: Vista Principal de Campaña (Estado Active)

**Modelo**: `crm.prospection`
**Tipo de Vista**: Formulario
**Registro**: Campaña "Q1 2025 - Sector Industrial" (external ID: `__test__campana_q1_industrial`)
**Estado**: Active

**Elementos a mostrar**:
- Encabezado con nombre de campaña
- Statusbar con estado "Active"
- Botones de acción: "Generate Leads", "Finish", "Cancel"
- Smart buttons con contadores:
  - Total Leads (15)
  - Total Pending (10)
  - Total Won (4)
  - Total Lost (1)
- Sección "Configuración de Campaña" con campos:
  - Modelo
  - Dominio
  - Método de asignación: Equitativamente
  - Usuarios asignados (Ana, Carlos, David)
- Sección "Leads/Oportunidades" con:
  - Tipo: Lead
  - Equipo: Equipo Comercial Industrial
  - Etiquetas: Q1 2025, Prospección Activa, Sector Industrial

**Nombre archivo**: `01_form_crm_prospection_active.png`

---

### Captura 2: Lista de Campañas

**Modelo**: `crm.prospection`
**Tipo de Vista**: Lista
**Filtro**: Todas las campañas

**Columnas a mostrar**:
- Nombre
- Responsable
- Total Pending
- Total Won
- Total Lost
- Estado

**Decoraciones visuales**:
- Filas en azul para campañas Active
- Filas en gris para campañas Done

**Datos esperados**: 2 campañas visibles
- "Campaña Q1 2025 - Sector Industrial" (Active)
- "Reactivación Clientes Q4 2024" (Done)

**Nombre archivo**: `02_list_crm_prospection.png`

---

### Captura 3: Vista de Lead con Referencia a Campaña

**Modelo**: `crm.lead`
**Tipo de Vista**: Formulario
**Registro**: Lead "Campaña Q1 2025 - Sector Industrial - Industrias Metálicas del Norte S.L."

**Elementos a mostrar**:
- Nombre del lead (heredado de campaña)
- Cliente potencial vinculado
- Comercial asignado (Ana García)
- Equipo de ventas (Equipo Comercial Industrial)
- **Campo custom "Prospección"**: Link a la campaña que lo generó
- Etiquetas heredadas de la campaña
- Descripción heredada de la campaña
- Estado: Lead (pendiente)
- Probabilidad: 15%

**Nombre archivo**: `03_form_crm_lead_with_prospection.png`

---

### Captura 4: Smart Button "Total Pending" Abierto

**Modelo**: `crm.prospection`
**Tipo de Vista**: Formulario (con modal/vista filtrada abierta)
**Acción**: Hacer clic en Smart Button "Total Pending"

**Resultado esperado**:
- Vista de lista de leads filtrada mostrando solo los 10 leads pendientes
- Títul de la ventana: "Total Pending"
- Leads visibles deben estar en estado pendiente (probability < 100, active=True)

**Nombre archivo**: `04_smart_button_pending_leads.png`

---

### Captura 5: Campaña en Estado Done

**Modelo**: `crm.prospection`
**Tipo de Vista**: Formulario
**Registro**: Campaña "Reactivación Clientes Q4 2024" (external ID: `__test__campana_reactivacion_q4`)
**Estado**: Done

**Elementos a mostrar**:
- Ribbon azul con texto "Done"
- Statusbar con estado "Done"
- Botones de acción deshabilitados/ocultos
- Smart buttons con métricas finales:
  - Total Leads (10)
  - Total Pending (0) - todos trabajados
  - Total Won (6)
  - Total Lost (4)
- Tasa de conversión final: 60% (6/10)

**Nombre archivo**: `05_form_crm_prospection_done.png`

---

## Configuración de Captura

### Resolución
- **Desktop**: 1920x1080 (por defecto)
- Capturar solo viewport, no página completa

### Idioma
- **Español (es_ES)**

### Elementos a Ocultar
- Barra de debug (`.o_debug_manager`)
- Notificaciones/Toasts (`.o_notification`)
- Tooltips (`.o_tooltip`)

### Tiempos de Espera
- Esperar selector `.o_content`: 15 segundos
- Esperar animaciones: 2 segundos adicionales
- Timeout de navegación: 30 segundos

---

## Datos de Prueba Requeridos

Para generar estas capturas, se requieren los siguientes datos importados:

### Usuarios (res.users)
- Ana García
- Carlos Martínez
- David López
- María Sánchez (responsable)

### Equipos (crm.team)
- Equipo Comercial Industrial
- Equipo Comercial General

### Etiquetas (crm.tag)
- Q1 2025
- Prospección Activa
- Sector Industrial
- Reactivación
- Alta Prioridad

### Empresas (res.partner)
- 20 empresas B2B de diferentes sectores

### Campañas (crm.prospection)
- Campaña Q1 2025 - Sector Industrial (Active, 15 leads)
- Reactivación Clientes Q4 2024 (Done, 10 leads)

### Leads (crm.lead)
- 25 leads/oportunidades distribuidos entre las 2 campañas
- Estados variados: pendiente, ganado, perdido

**Dataset disponible en**: `data/indaws_crm_prospection/`

---

## Salida Esperada

```
screenshots/indaws_crm_prospection/
├── 01_form_crm_prospection_active.png
├── 02_list_crm_prospection.png
├── 03_form_crm_lead_with_prospection.png
├── 04_smart_button_pending_leads.png
├── 05_form_crm_prospection_done.png
└── README_screenshots.md
```

**Total**: 5 screenshots documentando las funcionalidades principales del módulo.

---

## Notas Adicionales

- Las capturas 1, 2, 3 y 5 son obligatorias (vistas principales)
- La captura 4 es opcional pero recomendada (muestra funcionalidad de smart buttons)
- Si el usuario tiene datos adicionales, se pueden generar más capturas:
  - Vista kanban de leads (si existe)
  - Vista de estadísticas/gráficos (si existe)
  - Diferentes estados de campaña (Draft)
  - Proceso de cancelación de campaña

---

## Cómo Usar Esta Especificación

Este documento sirve como:

1. **Guía para el skill**: El skill `odoo-screenshot-generator` lee documentación similar y extrae automáticamente qué capturas generar

2. **Checklist manual**: Si generas capturas manualmente, usa esto como guía de qué capturar

3. **Template**: Copia y adapta este formato para otros módulos

Para generar estas capturas automáticamente:

```bash
/odoo-screenshot-generator /path/to/indaws_crm_prospection_documentation.md
```

El skill leerá la documentación del módulo (que contiene información similar a este documento) y generará las capturas automáticamente.
