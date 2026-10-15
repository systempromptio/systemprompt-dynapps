# project-validation-drafting: Workflow Details

## MODO PLANIFICAR (pre-proyecto)

Se ejecuta al inicio del proyecto, con analisis funcional pero sin implementacion.

### Paso 0: Recopilacion (STOP)

**Datos obligatorios:**
1. **ID del proyecto** en Odoo corporativo
2. **Credenciales Odoo corporativo** (URL, DB, API Key, usuario)

**DETENTE** y espera respuesta.

### Paso 1: Lectura Automatica de Odoo

```bash
./scripts/fetch_validation_context.sh <project_id> --pilot-dir /path/to/odoo-pilot/scripts
```

**Obtiene**:
a) Proyecto: nombre, partner_id, fechas, x_studio_knowledge
b) Articulos de analisis: hijos de "Analisis" bajo el articulo raiz
c) Tareas: nombre, descripcion, etapa, fechas, tags, horas
d) Requests existentes: x_client_request ya creados (evitar duplicados)

**Validaciones**: proyecto existe con x_studio_knowledge, articulo "Analisis" existe con hijos, tareas tienen fechas.

### Paso 2: Identificacion Automatica de Validaciones

Cruza analisis con tareas para identificar validaciones necesarias:

**Tipo APP**: 1 por cada articulo hijo de "Analisis" -> vincula a tarea de configuracion
**Tipo MIGRACION**: Si existen tareas [MIGR] -> 1 validacion, vincula a tarea principal
**Tipo DESARROLLO**: Tareas [DEV]/[INT] de peso >= 4 o integracion -> 1 por cada
**Tipo PREPRODUCCION**: Siempre exactamente 1 -> deadline = Go-Live - 5 dias laborables

Deadlines coherentes con planificacion. Consulta `./references/tipos_validacion.md` para reglas.

Presentar tabla al consultor.

**DETENTE** -- VALIDACION OBLIGATORIA. Consultor aprueba, elimina, anade o ajusta.

### Paso 3: Creacion de x_client_request en Draft

Por cada validacion aprobada, crear registro con:
- x_name: "[VALIDACION] {titulo}"
- x_type: "validation"
- x_priority: "1"
- x_state: "draft"
- x_deadline, x_task_id, x_partner_id, x_description (HTML base)

Descripcion base generada con template `./templates/descripcion_base.md` aplicando `./references/reglas_redaccion_cliente.md`.

**FIN modo PLANIFICAR.** Sugerir modo DOCUMENTAR para despues.

---

## MODO DOCUMENTAR (post-implementacion)

Se ejecuta cuando app/migracion/desarrollo esta completado.

### Paso 0: Recopilacion (STOP)

**Datos obligatorios:**
1. **ID del x_client_request** a documentar
2. **Credenciales Odoo corporativo** (si no configuradas)
3. **Credenciales Odoo del cliente** (opcional, para screenshots)

**DETENTE** y espera respuesta.

### Paso 1: Lectura de Contexto

Lee del Odoo corporativo:
- x_client_request con descripcion y tarea vinculada
- Tarea con checklists completados
- Articulo de analisis de la app
- Datos del proyecto y cliente

### Paso 2: Generacion del Documento

Template: `./templates/documento_validacion.md`
Reglas: `./references/reglas_redaccion_cliente.md`

6 secciones:
1. **Que se ha entregado**: Lenguaje de negocio, resultados tangibles
2. **Como funciona ahora**: Flujo desde perspectiva usuario final, con ejemplos reales
3. **Criterios de aceptacion**: Lista verificable y concreta
4. **Evidencia visual** (opcional): Placeholders para screenshots
5. **Limitaciones y aclaraciones**: Lo NO incluido, fases posteriores
6. **Aprobacion**: Aprobado / Con observaciones / No aprobado

**DETENTE** -- VALIDACION OBLIGATORIA. Consultor revisa antes de actualizar Odoo.

### Paso 3: Entregables

a) Markdown: `validacion_{tipo}_{nombre_slug}_{fecha}.md`
b) Actualizar x_client_request.x_description con HTML (estilos inline)
c) Knowledge article (opcional, si el consultor lo pide)

**FIN modo DOCUMENTAR.** Sugerir client-request-manager FOLLOW-UP para envio.
