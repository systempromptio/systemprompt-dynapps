# Requisitos Minimos para Tareas de Migracion de Modulos

## Contexto

Cada modulo que se migra entre versiones de Odoo debe pasar 5 gates antes de considerarse "done".
El redactor incluye estos gates como criterios de aceptacion en la tarea.

Filosofia: **migrar no es "que compile", es que funcione igual o mejor con datos reales.**

Un modulo sin tests antes de migrar es una bomba — no puedes saber si rompiste algo si no tenias
forma de verificarlo antes. Por eso el Gate 1 exige tests previos.

---

## Gate 1 — Preparacion (antes de tocar codigo)

### 1.1 Analisis de impacto

- Listar APIs deprecated/removed entre version origen y destino
  - Consultar `odoo-version-api-reference`
- Listar dependencias del modulo que cambian (renombradas, eliminadas, absorbidas)
- Identificar campos que cambian de tipo o comportamiento

### 1.2 Decision documentada

Cada modulo debe tener una decision explicita antes de invertir horas:

| Decision | Criterio |
|----------|----------|
| **Migrar** | Logica de negocio unica, sin equivalente en estandar/Core |
| **Eliminar** | Existe equivalente en la version destino o en Core |
| **Reemplazar** | Core/estandar cubre el 80%+, custom solo para el delta |
| **Refactorizar** | Migrar + aprovechar para limpiar deuda tecnica identificada |

### 1.3 Tests baseline en version origen

Si el modulo NO tiene tests en la version actual:

1. **Escribir tests en la version origen ANTES de migrar**
2. Cubrir flujos de negocio principales (no solo CRUD)
3. Verificar que pasan en verde en la version origen
4. Estos tests son la red de seguridad

No es opcional. "Funciona en mi staging" no es un criterio de aceptacion.

---

## Gate 2 — Migration Scripts

Scripts que transforman datos durante la migracion de BD. Van en `migrations/{version}/`.

### Estructura

```
migrations/
└── 19.0.1.0.0/
    ├── pre-migrate.py    # Antes de que ORM actualice el schema
    ├── post-migrate.py   # Despues de actualizar schema, antes de recomputar
    └── end-migrate.py    # Al final, despues de recomputar stored fields
```

### Cuando se necesita cada uno

| Script | Cuando usarlo | Ejemplo |
|--------|--------------|---------|
| `pre-migrate` | Renombrar columnas/tablas antes de que ORM las borre, backup de datos | Columna `name` pasa de Char a compute → salvar en columna temporal |
| `post-migrate` | Transformar datos al nuevo schema, actualizar XML IDs | Migrar valores de campo eliminado a campo nuevo |
| `end-migrate` | Validaciones finales, recomputar stored fields, limpieza | Verificar integridad referencial |

### Requisitos de los scripts

- **Idempotentes**: ejecutar 2 veces produce el mismo resultado
- **Logging**: `_logger.info(...)` en cada paso critico
- **Sin ORM en pre-migrate**: solo SQL directo
- **Transaccionales**: si falla un paso, falla todo el script
- **Documentados**: comentario al inicio explicando QUE migra y POR QUE

### Cuando NO se necesitan scripts

- Solo cambios de API Python → no hay migracion de datos
- Campos no cambian de nombre ni tipo → ORM maneja schema automaticamente
- Solo cambian vistas XML → no hay datos que migrar

En estos casos, documentar: "No se necesitan migration scripts porque [razon]".

---

## Gate 3 — Codigo Migrado

### Manifest (`__manifest__.py`)
- [ ] Version actualizada: `{version_destino}.1.0.0`
- [ ] Dependencias revisadas
- [ ] `license` presente
- [ ] `auto_install` solo en bridge modules

### Python
- [ ] APIs deprecated reemplazadas (consultar `odoo-version-api-reference`):
  - `name_get()` → `_compute_display_name` (v17+)
  - `_read_group()` nueva signature (v17+)
  - `fields_view_get()` → `get_views()` (v17+)
  - `_setup_complete()` eliminado (v18+)
- [ ] Herencia: `_inherit` correcta, `super()` sin romper cadena
- [ ] Sin imports obsoletos

### Vistas XML
- [ ] XPath compatible con estructura de la version destino
- [ ] Grupos de seguridad: verificar XML IDs
- [ ] Formularios: adaptados a cambios de layout
- [ ] Campos deprecated eliminados de las vistas

### JavaScript/OWL (si aplica)
- [ ] Componentes OWL compatibles con la version destino
- [ ] Servicios Odoo: API de registros, RPCs
- [ ] Assets: adaptados al nuevo sistema (`ir.asset` vs manifest)

---

## Gate 4 — Tests

### Minimos obligatorios

| Tipo de test | Que cubre | Cuando es obligatorio |
|-------------|-----------|----------------------|
| **Unitarios** | Logica de negocio de cada modelo con custom | Siempre |
| **Migracion de datos** | Los scripts de Gate 2 transforman correctamente | Cuando hay migration scripts |
| **Regresion** | Los flujos que funcionaban en origen siguen funcionando | Siempre |
| **Instalacion limpia** | El modulo instala sin errores en BD vacia | Siempre |

### Cobertura

- Todos los modelos con logica custom tienen al menos 1 test
- Flujos de negocio principales con test end-to-end (o al menos Form-based)
- Los migration scripts tienen tests que verifican la transformacion

---

## Gate 5 — Validacion Final

### Instalacion
- [ ] Instala limpio en BD vacia de la version destino
- [ ] Funciona en BD migrada desde la version origen

### Calidad
- [ ] Code review pasado
- [ ] `quality-gate` ejecutado sin Blockers ni Criticals
- [ ] Sin `# TODO: migrate` ni hacks temporales

### Datos
- [ ] Datos migrados verificados por muestreo
- [ ] Campos computed stored: recomputados y valores correctos
- [ ] Integridad referencial: sin Many2one apuntando a inexistentes

### Documentacion minima
- [ ] CHANGELOG o nota en commit describiendo que se migro
- [ ] Migration scripts documentados
- [ ] Decisiones tecnicas relevantes documentadas

---

## Caso especial: version destino no publicada

Cuando la version destino aun no esta disponible (ej: migrar v18 → v19 antes del release):

### Preparacion anticipada
1. **Escribir tests en la version actual** — la inversion mas valiosa
2. **Auditar modulos**: inventariar, clasificar, decidir migrar/eliminar/reemplazar
3. **Reducir deuda tecnica**: limpiar codigo, eliminar workarounds
4. **Documentar logica de negocio**: lo que no esta en tests ni en codigo

### Tracking de breaking changes
- Seguir runbot nightly builds, release notes, changelogs del repo destino
- Marcar modulos afectados por cada breaking change conocido

### Cuando la version se publique
1. Ejecutar tests baseline contra la nueva version → fallos indican que migrar
2. Priorizar por criticidad de negocio, no por facilidad tecnica
3. Aplicar los 5 gates normalmente

---

## Checklist rapido para el Redactor

- [ ] **G1**: Analisis de impacto documentado? Decision migrar/eliminar/reemplazar justificada?
- [ ] **G1**: Tests baseline existian o se crearon antes de migrar?
- [ ] **G2**: Migration scripts necesarios? Si si, idempotentes y documentados?
- [ ] **G2**: Si no se necesitan scripts, esta justificado?
- [ ] **G3**: Manifest, Python, XML, JS adaptados a la version destino?
- [ ] **G3**: APIs deprecated eliminadas?
- [ ] **G4**: Tests unitarios + regresion + migracion datos pasan en verde?
- [ ] **G4**: Cada modelo con logica custom tiene al menos 1 test?
- [ ] **G5**: Instala limpio en BD vacia Y funciona en BD migrada?
- [ ] **G5**: Code review + `quality-gate` sin Blockers?
- [ ] **G5**: Sin TODOs temporales ni hacks de migracion en el codigo?
