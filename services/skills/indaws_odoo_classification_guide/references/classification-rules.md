# Arboles de Decision por Contexto

## Arbol 1: Evaluacion de nueva funcionalidad

Usar cuando se evalua si hace falta desarrollo para una solicitud nueva (invocado por dev-assessment).

```
Funcionalidad solicitada
│
├─ GOLDEN RULES (evaluar PRIMERO)
│  ├─ Es Import/Export? → 🟣 Core (indaws_data_import/export) [OBLIGATORIO]
│  └─ Es Dashboard/KPI? → 🟢 Standard (Spreadsheets) [OBLIGATORIO]
│
├─ NIVEL 1: STANDARD
│  ├─ Consultar changelogs de la version del cliente
│  ├─ Usar conocimiento del LLM sobre Odoo estandar
│  └─ La funcionalidad existe 100% nativa? → 🟢 Standard
│
├─ NIVEL 2: CORE
│  ├─ Buscar en indices de modulos Indaws Core
│  └─ Existe modulo Core que lo cubra? → 🟣 Core [OBLIGATORIO si existe]
│
├─ NIVEL 3: STUDIO
│  ├─ Consultar studio-capabilities.md para la version del cliente
│  ├─ Cumple TODOS los criterios Studio? → 🟡 Studio
│  └─ Excede algun umbral? → Continuar a Custom
│
└─ NIVEL 4: CUSTOM
   └─ Clasificar por complejidad (ver complexity-thresholds.md)
      ├─ Menor (<200 LOC, ≤1 modelo) → 🔴 Custom Menor
      ├─ Estandar (200-800 LOC, 1-3 modelos) → 🔴 Custom Estandar
      └─ Complejo (>800 LOC, >3 modelos, JS/API) → 🔴 Custom Complejo
```

## Arbol 2: Migracion con refactoring completo

Usar cuando se clasifica un modulo existente para migracion (invocado por odoo-database-analyzer, estrategia refactoring).

```
Modulo a migrar
│
├─ GOLDEN RULES (evaluar PRIMERO)
│  ├─ Es Import/Export? → 🟣 Core [OBLIGATORIO]
│  └─ Es Dashboard/KPI? → 🟢 Standard (Spreadsheets) [OBLIGATORIO]
│
├─ NIVEL 1: STANDARD
│  ├─ Consultar Release Notes de la version destino
│  └─ Funcionalidad ahora 100% nativa? → 🟢 Standard
│
├─ NIVEL 2: CORE
│  ├─ Buscar en repositorios Indaws Core
│  └─ Existe modulo Core similar? → 🟣 Core [OBLIGATORIO si existe]
│
├─ NIVEL 3: STUDIO
│  ├─ Es personalizacion ligera? (<50 lineas logica)
│  └─ Campos, vistas, emails, automatizaciones simples? → 🟡 Studio
│
└─ NIVEL 4: CUSTOM
   └─ Logica compleja inevitable → 🔴 Custom
```

## Arbol 3: Migracion 1:1 (directa)

Usar cuando se migra un modulo sin refactoring (invocado por odoo-database-analyzer, estrategia 1:1).

```
Modulo a migrar
│
├─ Funcionalidad 100% identica a nativa en version destino?
│  ├─ SI (< 10 lineas Python custom) → 🟢 Standard
│  └─ NO → 🔴 Custom (DEFAULT)
│
└─ CLASIFICACIONES PROHIBIDAS:
   ❌ Studio - Nunca en 1:1
   ❌ Core - Nunca en 1:1
```

## Arbol 4: Migracion sin Studio

Usar cuando el cliente no tiene licencia Studio (invocado por odoo-database-analyzer, estrategia no-studio).

```
Modulo a migrar
│
├─ GOLDEN RULES (aplican igual)
│  ├─ Es Import/Export? → 🟣 Core [OBLIGATORIO]
│  └─ Es Dashboard/KPI? → 🟢 Standard [OBLIGATORIO]
│
├─ NIVEL 1: STANDARD → 🟢 Standard
├─ NIVEL 2: CORE → 🟣 Core
└─ NIVEL 3: CUSTOM (absorbe Studio) → 🔴 Custom
│
└─ CLASIFICACIONES PROHIBIDAS:
   ❌ Studio - No disponible
```

## Funcionalidades Odoo que los usuarios suelen desconocer

Antes de clasificar como Custom, verificar si alguna de estas cubre el requisito:

- Acciones de servidor configurables (sin codigo)
- Acciones automatizadas con condiciones (sin codigo)
- Campos computados con formula (mecanismo `ir.model.fields.compute` desde 14.0; en que version lo
  expone la UI de Studio esta **sin verificar** — ver `studio-capabilities.md`)
- Reglas de precios avanzadas (listas de precios multinivel)
- Portal del cliente/proveedor personalizable
- Reporting con filtros y agrupaciones avanzadas
- Aprobaciones de Studio: reglas por grupo con dominio desde 14.0 (`studio.approval.rule`);
  aprobadores nominales y delegacion desde 18.0
- Integracion con firma electronica (Enterprise)
- Spreadsheets con formulas y graficos (Enterprise)
- Consolidacion multi-empresa nativa

## Casos Frontera

### Studio con Python basico (disponible desde 14.0)
- SI aplica: asignar valores, crear registros simples, enviar emails con condiciones (<30 lineas)
- NO aplica: herencia de modelos, override de metodos, compute fields con logica compleja

### Core + extension (modulo bridge)
- Si Core cubre 80%+ y gap es simple (<100 lineas): crear modulo bridge que extienda Core
- Si gap es complejo: evaluar desde cero
- Consultar con equipo Core si el requisito faltante podria añadirse al modulo existente

### Migrabilidad como factor
- Modulo bien escrito con tests > configuracion Studio compleja
- Studio es preferible para cambios temporales o frecuentes
- Desarrollo es preferible para logica de negocio critica y estable
- Para versiones < 16.0, preferir desarrollo. **Es una recomendacion, no un limite tecnico**: campos
  computados, Python en automatizaciones, approvals y crear apps ya existen en 14.0 (citas en
  `studio-capabilities.md`). Lo que si falta verificado en 14.0/15.0 son los **dashboards**
  (`spreadsheet_dashboard` llega en 16.0). El motivo real de la recomendacion es que esas versiones
  estan fuera de soporte y toda configuracion Studio hay que rehacerla al migrar.
