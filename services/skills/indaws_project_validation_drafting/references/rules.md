# project-validation-drafting: Rules and Guidelines

## Writing Rules

- **CERO tecnicismos**: Consulta `./references/reglas_redaccion_cliente.md` para diccionario completo
- **Ejemplos concretos**: Siempre con datos reales del entorno del cliente
- **Orientado a resultados**: "Que puede hacer ahora", no "que hemos configurado"
- **Tono**: Profesional, cercano, en segunda persona (tu/vosotros)

## Client Request Integration

- Requests con `x_type: "validation"` y `x_priority: "1"` (siempre bloqueante)
- Titulo SIEMPRE empieza con `[VALIDACION]`
- Estado inicial SIEMPRE `draft` -- envio lo gestiona client-request-manager FOLLOW-UP
- NUNCA enviar emails desde este skill

## Deadline Rules

- Coherentes con planificacion del proyecto
- Apps: alineado con fecha fin de tarea de configuracion
- Migracion: alineado con fecha fin de tarea de migracion
- Desarrollos: alineado con fecha fin de tarea de desarrollo
- Pre-produccion: Go-Live menos 5 dias laborables
- Si tarea sin fechas: avisar y pedir fecha manual

## No Duplicates

- Verificar que no exista request con titulo similar antes de crear
- Si ya existe, preguntar: actualizar existente o crear nuevo

## Validations

**Interactivas (obligatorias)**:
1. Validacion de lista (Paso 2 PLANIFICAR)
2. Validacion de documento (Paso 2 DOCUMENTAR)

**Automaticas**:
3. Proyecto con x_studio_knowledge valido
4. Articulo "Analisis" con hijos
5. No se crean duplicados
6. Descripciones usan lenguaje de negocio (sin tecnicismos prohibidos)
7. Deadlines coherentes (no antes de hoy, no despues de Go-Live)
8. Criterios de aceptacion concretos y verificables
9. HTML con estilos inline

## Success Criteria

- Validaciones identificadas correctamente del analisis y tareas
- Consultor ha validado lista (PLANIFICAR)
- x_client_request creados con [VALIDACION], tipo validation, deadlines coherentes
- Documento usa exclusivamente lenguaje de negocio
- Criterios concretos, verificables, personalizados
- Consultor ha validado documento (DOCUMENTAR)
- Entregables generados correctamente
