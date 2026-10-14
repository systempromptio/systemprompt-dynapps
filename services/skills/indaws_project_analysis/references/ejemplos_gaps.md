# Ejemplos de GAPs Comunes por Módulo de Odoo

Este documento contiene ejemplos de GAPs frecuentes que suelen aparecer en proyectos de implantación de Odoo, organizados por módulo.

**Uso**: Consulta este documento para inspirarte al detectar GAPs en el cuestionario del cliente y para verificar si has cubierto los casos más comunes.

---

## CRM

### GAPs Comunes

| Requisito Cliente | Odoo Estándar | GAP | Solución Típica |
|-------------------|---------------|-----|-----------------|
| Múltiples pipelines de venta por línea de negocio | ❌ Solo permite 1 pipeline (etapas) por equipo | Falta flexibilidad para gestionar diferentes procesos comerciales | [DEV] Módulo custom de pipelines múltiples |
| Campos personalizados masivos en oportunidades (>20 campos) | ✓ Soporta campos custom | ⚠️ Gestión masiva de campos puede requerir vistas optimizadas | [STUDIO/AUTO] + [DEV] para vistas optimizadas |
| Integración con sistema telefónico para registrar llamadas | ❌ No tiene integración nativa con VoIP | Falta captura automática de llamadas | [CORE] si existe conector o [DEV] integración API |
| Scoring automático de leads | ❌ No tiene sistema de scoring | Falta priorización automática | [STUDIO/AUTO] reglas simples o [DEV] si es complejo |

---

## Ventas

### GAPs Comunes

| Requisito Cliente | Odoo Estándar | GAP | Solución Típica |
|-------------------|---------------|-----|-----------------|
| Múltiples descuentos en cascada (3+) | ❌ Solo permite 1 descuento por línea | Falta cálculo de descuentos secuenciales | [CORE] Gestión de Descuentos Múltiples |
| Aprobación multinivel de presupuestos | ❌ No tiene flujo de aprobación | Falta control de autorización | [CORE] Flujo de aprobación de ventas |
| Configurador de producto complejo | ⚠️ Tiene variantes pero limitado | Falta configuración dinámica con reglas complejas | [DEV] Configurador personalizado |
| Tarifas con fórmulas dinámicas complejas | ⚠️ Tarifas básicas soportadas | Falta cálculo con múltiples variables | [DEV] Motor de tarificación custom |
| PDFs de presupuesto con diseño corporativo específico | ⚠️ Reportes QWeb personalizables pero limitados | Diseño muy específico no alcanzable con QWeb estándar | [CORE] Generador de PDFs o [DEV] |

---

## Compras

### GAPs Comunes

| Requisito Cliente | Odoo Estándar | GAP | Solución Típica |
|-------------------|---------------|-----|-----------------|
| Aprobación multinivel de pedidos de compra | ❌ No tiene flujo de aprobación nativo | Falta control de autorización por importe | [STUDIO/AUTO] si es simple o [DEV] si complejo |
| Gestión de contratos marco con proveedores | ⚠️ Acuerdos de compra limitados | Falta gestión avanzada de contratos | [DEV] Módulo de contratos |
| Comparativa automática de 3 presupuestos | ❌ No tiene comparativa automática | Falta análisis comparativo | [DEV] Módulo de comparativa |
| Recepción parcial con calidad | ✓ Soporta recepciones parciales | ⚠️ Falta integración con control de calidad | [STUDIO/AUTO] + Control de Calidad |

---

## Inventario

### GAPs Comunes

| Requisito Cliente | Odoo Estándar | GAP | Solución Típica |
|-------------------|---------------|-----|-----------------|
| Trazabilidad por números de serie con campos adicionales | ✓ Trazabilidad estándar | ⚠️ Faltan campos custom en lotes/series | [STUDIO/AUTO] Campos personalizados |
| Ubicaciones con capacidad máxima | ❌ No gestiona capacidad de ubicaciones | Falta control de espacio | [DEV] Módulo de gestión de capacidad |
| Re-aprovisionamiento con fórmulas complejas | ⚠️ Reglas básicas de stock | Falta cálculo avanzado de necesidades | [DEV] Motor de aprovisionamiento custom |
| Inventario cíclico automatizado | ❌ No tiene inventario cíclico | Falta automatización de conteos | [DEV] Módulo de inventario cíclico |
| Gestión de caducidades con alertas | ⚠️ Soporta fechas de caducidad | Falta sistema de alertas proactivo | [STUDIO/AUTO] Automated actions |

---

## Facturación y Contabilidad

### GAPs Comunes

| Requisito Cliente | Odoo Estándar | GAP | Solución Típica |
|-------------------|---------------|-----|-----------------|
| OCR para digitalización de facturas | ❌ No tiene OCR nativo | Falta captura automática | [CORE] Agente IA: OCR para facturas |
| Gestión de IPC en contratos | ❌ No gestiona IPC | Falta actualización automática por inflación | [CORE] Gestión de IPC |
| Envío automático a SII (España) | ❌ No incluido en versión CE | Falta cumplimiento legal | [CORE] SII o módulo de localización |
| VeriFacTu (España 2025) | ❌ No incluido | Falta cumplimiento legal obligatorio | [CORE] VeriFacTu |
| Conciliación bancaria con reglas ML | ⚠️ Conciliación básica | Falta aprendizaje automático | [DEV] Motor de conciliación con ML |
| Múltiples cuentas analíticas por línea | ❌ Solo 1 cuenta analítica | Falta distribución multi-dimensional | [DEV] Extensión de analítica |

---

## Recursos Humanos

### GAPs Comunes

| Requisito Cliente | Odoo Estándar | GAP | Solución Típica |
|-------------------|---------------|-----|-----------------|
| Portal de empleado sin usuario Odoo | ❌ Requiere usuario en Odoo | Falta acceso externo económico | [CORE] Portal de empleado |
| Fichaje por proyecto/tarea | ❌ No vincula asistencias a proyectos | Falta imputación de horas | [CORE] Fichaje por Proyecto |
| Gestión de turnos rotativos | ❌ No gestiona turnos | Falta planificación de turnos | [DEV] Módulo de turnos |
| Nóminas con convenio específico | ⚠️ Nóminas básicas | Falta cálculo según convenio español | [DEV] Adaptación de nóminas |
| Gestión de dietas y gastos con geolocalización | ⚠️ Gastos básicos | Falta validación automática | [STUDIO/AUTO] + [DEV] para geolocalización |

---

## Fabricación

### GAPs Comunes

| Requisito Cliente | Odoo Estándar | GAP | Solución Típica |
|-------------------|---------------|-----|-----------------|
| BOM con variantes dinámicas | ⚠️ BOM con variantes limitado | Falta generación dinámica de componentes | [DEV] Configurador de BOM |
| Planificación con capacidad finita | ❌ Planificación básica sin capacidad | Falta optimización de recursos | [DEV] Motor de planificación APS |
| Trazabilidad inversa (de producto final a lotes) | ⚠️ Trazabilidad básica | Falta reporting de trazabilidad inversa | [DEV] Informes de trazabilidad |
| Gestión de scrap con motivos y costes | ⚠️ Scrap básico | Falta análisis de desperdicios | [STUDIO/AUTO] Campos + informes |
| Órdenes de fabricación con subcontratación compleja | ⚠️ Subcontratación básica | Falta gestión avanzada de subcontratas | [DEV] Módulo de subcontratación |

---

## Proyectos

### GAPs Comunes

| Requisito Cliente | Odoo Estándar | GAP | Solución Típica |
|-------------------|---------------|-----|-----------------|
| Gestión de materiales de proyecto facturables | ❌ No vincula stock a proyectos | Falta trazabilidad de materiales | [CORE] Materiales de Proyecto |
| Timesheet con aprobación | ⚠️ Timesheet básico | Falta flujo de aprobación | [STUDIO/AUTO] Automated actions |
| Facturación por hitos | ⚠️ Facturación básica | Falta gestión de hitos complejos | [DEV] Módulo de facturación por hitos |
| Gestión de servicios de campo (FSM) | ❌ No incluye FSM en CE | Falta gestión de intervenciones | [DEV] Módulo FSM custom |
| Planificación de recursos (Gantt avanzado) | ⚠️ Gantt básico | Falta planificación de capacidad | [DEV] Extensión de Gantt |

---

## TPV (Punto de Venta)

### GAPs Comunes

| Requisito Cliente | Odoo Estándar | GAP | Solución Típica |
|-------------------|---------------|-----|-----------------|
| Integración con terminal de pago (Paytef, etc.) | ❌ No incluye integraciones nativas | Falta conexión con TPV físico | [CORE] Payment Paytef o similar |
| Gestión de cajas con múltiples turnos | ⚠️ Gestión de caja básica | Falta control de turnos rotativos | [DEV] Extensión de caja |
| Programa de fidelización en TPV | ❌ Módulo separado | Falta integración fluida | [CORE] Loyalty o integración custom |
| Tickets personalizados con promociones | ⚠️ Tickets básicos | Falta diseño avanzado | [DEV] Personalización de tickets |
| Sincronización offline robusta | ⚠️ Offline básico | Falta gestión de conflictos | [DEV] Mejora de sincronización |

---

## E-commerce

### GAPs Comunes

| Requisito Cliente | Odoo Estándar | GAP | Solución Típica |
|-------------------|---------------|-----|-----------------|
| Integración con Prestashop/WooCommerce | ❌ No incluye conectores | Falta sincronización bidireccional | [CORE] Conector Prestashop o [DEV] |
| Portal B2B con precios por cliente | ⚠️ Portal básico | Falta gestión de tarifas personalizadas | [CORE] Portal B2B o [DEV] |
| Configurador de producto en web | ❌ No incluye configurador web | Falta interactividad | [DEV] Configurador web |
| Feeds de productos para Google Shopping | ❌ No genera feeds | Falta exportación automatizada | [CORE] Product Data Feed o [DEV] |
| Sistema de reseñas con moderación | ⚠️ Reseñas básicas | Falta moderación y gestión | [STUDIO/AUTO] + [DEV] |

---

## Notas Finales

- Estos son ejemplos basados en experiencia real de proyectos
- Cada cliente es único - siempre analiza sus requisitos específicos
- Verifica SIEMPRE la versión de Odoo, ya que funcionalidades cambian entre versiones
- Antes de sugerir [DEV], consulta si existe paquete [CORE] que resuelva el GAP
- Los GAPs marcados con ⚠️ indican que Odoo tiene funcionalidad pero limitada
- Los GAPs marcados con ❌ indican ausencia total de funcionalidad
