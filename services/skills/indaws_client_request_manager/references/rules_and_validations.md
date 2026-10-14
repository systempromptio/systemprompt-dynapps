# Client Request Manager - Rules and Validations

## Rules

### Generales

- SIEMPRE pide los datos obligatorios antes de empezar (Paso 0)
- SIEMPRE presenta la lista de peticiones propuestas al consultor antes de crear nada
- SIEMPRE presenta el borrador de email antes de enviar
- NUNCA envies emails sin validacion del consultor
- NUNCA dupliques peticiones que ya existen para la misma tarea
- NUNCA inventes peticiones genericas -- cada peticion debe estar justificada por el contexto
- SIEMPRE incluye en la descripcion del request: que se necesita, por que, y en que formato
- SIEMPRE coteja contra el analisis funcional antes de proponer peticiones (modo GENERAR)
- SIEMPRE redacta descripciones en segunda persona (tu/vosotros), tono cercano pero profesional
- NUNCA solicites acceso al software o aplicaciones del cliente (Redmine, SAP, Sage, etc.). Los datos para migrar deben venir en Excel o CSV
- NUNCA solicites acceso a la banca online del cliente. La sincronizacion bancaria es self-service
- SIEMPRE calcula deadlines basandote en `planned_date_begin` de la tarea, NO en la fecha actual

### Cotejo con el Analisis

- Lee el documento de analisis COMPLETO antes de proponer peticiones
- Busca informacion relevante en TODAS las secciones (no solo la seccion de la app correspondiente)
- Si el analisis responde parcialmente, la peticion se mantiene pero se ajusta
- Muestra siempre las peticiones descartadas con la referencia a la seccion del analisis

### Emails

- Tono profesional, cordial y claro
- NUNCA mencionar detalles internos del proyecto (horas, presupuesto, herramientas internas)
- SIEMPRE incluir el deadline de forma amable ("Seria ideal recibirlo antes del 15 de marzo")
- Si la peticion es bloqueante, indicarlo con tacto
- Asunto del email: `[{Proyecto}] {Titulo de la peticion}`

### Conexion a Odoo

- Usa SIEMPRE los scripts `fetch_task_context.sh` y `fetch_requests.sh` como metodo principal
- Si el script no esta disponible o falla, usa los scripts de `odoo-pilot` directamente
- Verifica que `x_client_request` existe antes de intentar crear registros
- No asumas que todos los campos custom existen; maneja ausencias gracefully

## Validaciones

### Interactivas (obligatorias)

1. **Validacion de peticiones (Paso 2 GENERAR)**: Consultor aprueba lista de peticiones con justificacion de cotejo antes de crear
2. **Validacion de emails (Paso 2 FOLLOW-UP)**: Consultor aprueba borrador de cada email antes de enviar

### Automaticas

3. Tarea encontrada en Odoo con project_id valido
4. Proyecto tiene partner_id (necesario para emails)
5. Modelo x_client_request existe en la instancia
6. No se crean peticiones duplicadas (cotejo con requests existentes)
7. Toda peticion tiene justificacion (no generica)
8. Toda peticion descartada tiene referencia al analisis
9. Emails no contienen informacion interna (horas, presupuesto)
10. Deadlines son coherentes (no en el pasado, no excesivamente lejanos)
