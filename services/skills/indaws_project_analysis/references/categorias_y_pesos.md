# Definiciones de Categorías y Guía de Pesos

Este documento define todas las categorías de tareas utilizadas en el análisis funcional y la guía de asignación de pesos según complejidad y esfuerzo.

---

## Categorías de Tareas

| Categoría | Descripción | Ejemplos |
|-----------|-------------|----------|
| `[CONFIG]` | Ajustes estándar vía interfaz. Sin Studio ni código. | Activar módulos, configurar parámetros del sistema, crear usuarios y permisos, configurar tarifas estándar |
| `[STUDIO/AUTO]` | Odoo Studio, Acciones Automatizadas, Reglas de Servidor (sin Python complejo). | Campos personalizados, automated actions básicas, reglas de servidor con código simple, vistas personalizadas con Studio |
| `[CORE]` | Paquetes de Indaws Core. Módulos desarrollados por Indaws que extienden funcionalidades de Odoo. | Framework de API, Portal de empleado, Generador de PDFs, Flujo de aprobación de ventas, Gestión de Descuentos Múltiples |
| `[DEV]` | Código Python, modelos, herencias complejas, API, QWeb complejos, JS personalizado. | Nuevos modelos de datos, lógica de negocio compleja, integraciones con sistemas externos, informes QWeb avanzados |
| `[MIGR]` | Carga de datos desde otros sistemas. | Importación de maestros (clientes, productos), migración de transacciones históricas, carga de saldos iniciales |
| `[TEST]` | Pruebas funcionales y validación. Especificar escenarios clave. | Pruebas de flujos completos, validación de cálculos, tests de integraciones, UAT (User Acceptance Testing) |
| `[TRAIN]` | Formación a usuarios. Indicar grupo objetivo y funcionalidades a cubrir. | Formación a comerciales en CRM, capacitación de almacén en inventario, training de administradores |
| `[ANLS]` | Análisis funcional o técnico detallado específico (precursor de DEV complejo). | Análisis de integración detallado, estudio de viabilidad técnica, diseño de arquitectura de solución |

---

## Guía de Pesos (Complejidad/Esfuerzo)

| Peso | Descripción | Tiempo Estimado | Ejemplos |
|------|-------------|-----------------|----------|
| **1** | **Muy Simple** - Configuración trivial | minutos - 1h | Activar un módulo, crear un usuario, configurar un parámetro simple |
| **2** | **Simple** - Configuración estándar clara, Studio/Auto básico | 1-4h | Configurar tarifas, crear campos custom básicos, automated action simple |
| **3** | **Medio** - Configuración avanzada, Studio/Auto con lógica, Migración simple | 4h - 1 día | Migración de maestros (clientes/productos), Studio con múltiples campos y lógica, automated actions con condiciones |
| **4** | **Complejo** - DEV pequeño/mediano, Migración con transformaciones | 1-3 días | Desarrollo de módulo sencillo, migración con transformaciones de datos, integración básica con API externa |
| **5** | **Muy Complejo** - DEV extenso, Migración compleja | >3 días | Desarrollo de módulo complejo con múltiples modelos, integración compleja con sistema legacy, migración masiva con reglas de negocio complejas |

---

## Criterios para Asignar Pesos

### Para [CONFIG]:
- **Peso 1**: Configuración de 1 click o muy directa
- **Peso 2**: Requiere navegar por varios menús o configurar múltiples parámetros relacionados
- **Peso 3**: Configuración avanzada que requiere entender bien el negocio o afecta múltiples áreas

### Para [STUDIO/AUTO]:
- **Peso 2**: 1-3 campos custom o 1 automated action muy simple
- **Peso 3**: 4-10 campos custom o automated action con condiciones/lógica básica
- **Peso 4**: >10 campos custom (sin lógica compleja) o automated action de un paso sin Python. La **lógica compleja o el Python no trivial NO son [STUDIO/AUTO] → van a [DEV]** (criterio en `odoo-classification-guide`).

### Para [DEV]:
- **Peso 3**: Desarrollo trivial (< 100 líneas de código, 1 modelo simple, sin lógica compleja)
- **Peso 4**: Desarrollo pequeño/mediano (< 500 líneas, 1-2 modelos, lógica moderada)
- **Peso 5**: Desarrollo extenso (> 500 líneas, múltiples modelos, lógica compleja, integraciones)

### Para [MIGR]:
- **Peso 2**: Migración de < 100 registros sin transformaciones
- **Peso 3**: Migración de 100-1000 registros con transformaciones simples
- **Peso 4**: Migración de 1000-10000 registros con transformaciones complejas
- **Peso 5**: Migración de >10000 registros o con reglas de negocio muy complejas

### Para [TEST]:
- **Peso 1**: Test simple de 1 funcionalidad
- **Peso 2**: Test de flujo completo sin muchas variantes
- **Peso 3**: Test exhaustivo de múltiples escenarios o flujo completo end-to-end

### Para [TRAIN]:
- **Peso 2**: Formación básica de 1-2 horas a <5 usuarios
- **Peso 3**: Formación completa de 1 día a grupo de usuarios
- **Peso 4**: Formación extensiva multi-sesión a múltiples grupos

---

## Reglas de Uso en el Checklist

1. **Agrupación**: Agrupar por categoría en este orden: [ANLS] → [CONFIG] → [MIGR] → [STUDIO/AUTO] → [DEV] → [TEST] → [TRAIN]

2. **Orden dentro de categoría**: Secuencia lógica de implementación (no por peso)

3. **Exclusión de tareas detalladas**: Las tareas [DEV] que aparecen en la sección 7 (Tareas Detalladas) NO deben incluirse en el checklist de la sección 6

4. **Descripción clara**: Usar verbo de acción + objeto + contexto específico
   - ❌ MAL: "Configurar productos"
   - ✅ BIEN: "Configurar categorías de productos y atributos de variantes"

5. **Granularidad**: Cada tarea debe ser accionable y medible
   - ❌ MAL: "Implementar todo el módulo de ventas" (demasiado grande)
   - ✅ BIEN: Dividir en tareas específicas (configurar, migrar, desarrollar, testear)

---

## Ejemplo de Checklist Bien Formado

| Categoría | Tarea | Peso |
|-----------|-------|------|
| [ANLS] | Analizar integración con sistema de envíos externo | 3 |
| [CONFIG] | Activar módulos de Ventas, CRM e Inventario | 1 |
| [CONFIG] | Configurar categorías de productos y atributos de variantes | 2 |
| [CONFIG] | Crear usuarios y asignar permisos por rol | 2 |
| [MIGR] | Migrar clientes desde Excel (500 registros) | 3 |
| [MIGR] | Migrar productos con categorías y tarifas (1200 registros) | 3 |
| [STUDIO/AUTO] | Crear campos personalizados en ficha de cliente (7 campos) | 3 |
| [STUDIO/AUTO] | Crear automated action para envío de email al cerrar venta | 2 |
| [DEV] | Implementar cálculo de comisiones de vendedor | 3 |
| [TEST] | Validar flujo completo de venta (presupuesto → pedido → factura → cobro) | 3 |
| [TEST] | Validar cálculo de comisiones con múltiples escenarios | 2 |
| [TRAIN] | Formar a comerciales en uso de CRM y pipeline de ventas (5 usuarios, 4h) | 3 |
| [TRAIN] | Formar a administración en facturación y cobros (3 usuarios, 3h) | 2 |

---

## Notas Finales

- Los pesos son **estimaciones orientativas** basadas en experiencia
- Factores que pueden aumentar el peso:
  - Complejidad del negocio del cliente
  - Calidad de los datos a migrar
  - Número de integraciones con sistemas externos
  - Nivel de personalización requerido
- En caso de duda, es mejor **sobrestimar** ligeramente que subestimar
