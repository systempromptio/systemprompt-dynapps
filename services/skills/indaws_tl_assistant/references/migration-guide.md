# Guía TL Migraciones Odoo v17→v18→v19

## Principios

- Migración = reingeniería de procesos, no solo parche técnico
- Cada versión saltada = x1.5 complejidad
- Oportunidad de eliminar deuda: "¿puede reemplazarse por estándar?"
- Freeze código antes de solicitar BD test al Upgrade Service

## Estrategias

| Estrategia | Cuándo | Ventaja | Limitación |
|-----------|--------|---------|-----------|
| **Upgrade Service** | Datos históricos importantes, Enterprise | Migra datos/estructura auto | No migra módulos custom |
| **OpenUpgrade** | Community sin Enterprise | Control total, gratis | Puede ir por detrás en versiones |
| **Clean Slate** | Mucha deuda, datos inconsistentes | Limpio | Pierde histórico |

## 6 Fases

### 1. Auditoría Sistema Actual
Inventariar módulos (estándar/Enterprise/custom), evaluar calidad datos, mapear integraciones.
Skills: `/odoo-database-analyzer`, `/odoo-module-scanner`

### 2. Planificación
Decidir estrategia, priorizar módulos por criticidad, estimar con contingencia, rollback plan.
Skill: `/migration-assistant`

### 3. Migración Módulos Custom
Uno a uno, menor dependencia primero. Por módulo: manifest→modelos→vistas→JS→tests.
Skills: `/odoo-module-upgrader`, `/odoo-version-api-reference`

### 4. Test Migration (Iterativa)
Dump → Upgrade Service → fixes SQL → validar staging → repetir.
Skill: `/odoo-database-upgrader`

### 5. UAT
Validación usuarios reales por flujo de negocio. Skill: `/quality-gate`

### 6. Go-Live + Hyper-Care
Corte planificado, soporte intensivo 1-4 semanas, response <1h críticos.

## Decisiones Clave TL

| Decisión | Criterio |
|----------|---------|
| ¿Migrar o reinstalar? | Deuda >40% esfuerzo migración → considerar reinstalar |
| ¿Migrar custom o reemplazar? | Si existe en versión nueva → eliminar custom |
| ¿Migrar OCA? | Política Indaws: no OCA. Evaluar alternativa Core |
| ¿Directa o paso a paso? | Paso a paso más seguro; directa solo si Upgrade Service soporta |
| ¿Cuándo corte? | Viernes noche o finde largo; nunca cierre fiscal |

> **Template**: `templates/migration-assessment.md`
