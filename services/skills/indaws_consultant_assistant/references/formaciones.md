# Formaciones

## Qué es

La planificación, preparación y ejecución de formaciones para el equipo del cliente. Cada formación cubre una app o funcionalidad específica y se adapta al perfil del usuario.

## Cuándo aplica

- Fase 3 (Pre-proyecto): Planificación del calendario de formaciones
- Fase 4.2 (Ejecución): Preparación de materiales y ejecución de sesiones

## Qué debe saber el consultor

### 9 Perfiles de usuario

| Perfil | Nivel técnico | Apps típicas | Formato sesión |
|--------|--------------|--------------|----------------|
| Director general | Bajo | Dashboard, Informes | 30-45 min, demo |
| Director financiero | Medio | Contabilidad, Informes | 45-60 min, caso práctico |
| Resp. comercial | Medio | CRM, Ventas | 45-60 min, workflow completo |
| Resp. compras | Medio | Compras, Inventario | 45-60 min, workflow completo |
| Resp. almacén | Bajo-Medio | Inventario, MRP | 60 min, práctica hands-on |
| Resp. RRHH | Medio | RRHH, Ausencias, Gastos | 45-60 min, caso práctico |
| Administrativo | Bajo | Facturación, Documentos | 30-45 min, paso a paso |
| Técnico/Producción | Bajo | MRP, Mantenimiento | 45 min, práctica |
| Usuario estándar | Bajo | Portal, Firma | 20-30 min, guía rápida |

### Regla 50/50

Toda formación debe dividirse en:
- **50% teoría**: Explicación conceptual, flujos, reglas de negocio
- **50% práctica**: Ejercicios con datos reales del cliente, casos prácticos

### Regla de los 20 minutos

Cada bloque de contenido no debe superar 20 minutos sin interacción del usuario. Después de 20 minutos de teoría, incluir un ejercicio práctico.

### Planificación de formaciones

- **Al inicio del proyecto**: Se ejecuta `training-planner` en modo planificación para definir a alto nivel todas las formaciones previstas (apps, perfiles, fechas aproximadas).
- **Antes de cada formación concreta**: Se ejecuta `training-planner` de nuevo para preparar los materiales específicos de esa sesión.

El skill `training-planner` genera materiales para **UNA** formación concreta por cada invocación de preparación. La planificación inicial da una visión global del calendario formativo.

### Entregables por formación

1. **Plan de formación** (`{app}_plan_formacion.md`): Objetivos, agenda, duración
2. **Guía de sesión** (`{app}_guia_sesion.md`): Contenido detallado con capturas
3. **Cheat sheet** (`{app}_cheat_sheet.md`): Referencia rápida para el usuario

### Input principal: Tarea de Odoo

El skill lee la tarea de formación (ID) como fuente principal via odoo-pilot. La descripción de la tarea y el chatter contienen el contexto necesario.

## Qué debe hacer el consultor

1. Al inicio del proyecto, ejecutar `training-planner` para planificar a alto nivel todas las formaciones
2. Antes de cada sesión concreta, ejecutar `training-planner` de nuevo con el ID de la tarea
3. Revisar y personalizar los materiales generados
4. Impartir la formación con datos reales del cliente
5. Tras la reunión de formación, ejecutar `project-meeting-minutes` (genera el acta, detecta acciones pendientes y verifica el alcance)

## Skills relacionados

- `training-planner`: Skill principal (planificación global al inicio + materiales por sesión)
- `odoo-screenshot-generator`: Para generar capturas en los materiales
- `project-meeting-minutes`: Se ejecuta después de cada reunión de formación; genera el acta, detecta acciones y verifica el alcance

## Errores comunes

1. **Formación sin datos reales**: Los ejercicios con datos de demo no preparan al usuario
2. **Sesiones demasiado largas sin práctica**: Romper la regla de los 20 minutos pierde la atención
3. **No adaptar al perfil**: Un director no necesita el mismo nivel de detalle que un administrativo
4. **Formación antes de completar configuración**: El sistema debe estar configurado para que la formación tenga sentido
5. **No generar cheat sheet**: El usuario necesita una referencia rápida post-formación
