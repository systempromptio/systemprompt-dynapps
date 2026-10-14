# Template: Planificación Técnica

> Usar cuando Dev o TL preguntan: "¿Por dónde empiezo?", "Orden de migración", "Plan técnico"
> Dos modos: con API (usa analyze_dependencies.py) y sin API (criterios de references/phases.md).

---

## Plan Técnico — {cliente} v{origen} → v{destino}

### Orden de migración de módulos

#### Modo con API (datos del proyecto en Odoo)

> Ejecutar `scripts/analyze_task_dependencies.py` para obtener orden topológico.
> Presentar resultado con este formato:

| Orden | Módulo | Estrategia | Dependencias | Horas est. | Riesgo |
|-------|--------|-----------|-------------|-----------|--------|
| 1 | {modulo} | {estrategia} | — | {horas} | {riesgo} |
| 2 | {modulo} | {estrategia} | {dep1} | {horas} | {riesgo} |
| ... | | | | | |

#### Modo sin API (sin proyecto en Odoo)

> Aplicar criterios de ordenación de `references/phases.md`:
> 1. Módulos **Standard** primero (eliminar y usar nativo — quick wins)
> 2. Módulos **Core** y **Studio** después (instalar/configurar + scripts)
> 3. Módulos **Custom** al final (mayor superficie de cambio)
> *(Es orden de ejecución de la migración, no preferencia; el criterio Studio-vs-custom lo fija `odoo-classification-guide`.)*
> 4. Dentro de cada grupo: sin dependencias → con dependencias
> 5. Empate: menor riesgo primero

### Gestión de scripts

> Referenciar `references/script-management.md` para:
> - Estructura de directorios de scripts
> - Nomenclatura (NNN_descripcion.py)
> - Template de script con execute() y rollback()
> - Reglas de idempotencia y dry-run

### Entorno de trabajo

| Aspecto | Detalle |
|---------|---------|
| Rama | `release/migration-{version}` |
| Entorno testing | {url_testing} |
| Pipeline CI/CD | Pre-commit + Ruff + Pylint-Odoo + ESLint |
| Tests | Unitarios por módulo migrado |

### Protocolo de comunicación durante desarrollo

| De → A | Frecuencia | Formato |
|--------|-----------|---------|
| Dev → BC | 2x/semana | Informe Dev→BC (template `dev-to-bc-report.md`) |
| Dev → TL | Diario | Update breve en tarea Odoo |
| Decisiones funcionales | Inmediato | Escalar al BC — NO asumir respuestas |

---

## Reglas de formato

- Si hay proyecto en Odoo con tareas: usar `analyze_task_dependencies.py` y mostrar orden real
- Si no hay proyecto: usar criterios de ordenación manuales de phases.md
- Siempre incluir sección de scripts referenciando `references/script-management.md`
- Siempre incluir protocolo de comunicación de `references/communication.md`
- Para Dev: incluir referencia al skill `odoo-module-upgrader` para patrones de código
- Para TL: incluir puntos de review y validación de arquitectura
