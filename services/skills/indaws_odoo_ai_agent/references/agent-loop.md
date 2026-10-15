# Bucle del agente — Agent SDK dentro de Odoo (gate humano, streaming, resume)

> Patrones extraídos del build **web_shell** (Claude en Odoo v19, 2026-07). El conocimiento
> genérico del Agent SDK vive en la skill `claude-api` / agente `claude-code-guide`; aquí solo la
> integración con Odoo. **Verifica las firmas exactas del SDK contra su fuente/docs antes de
> emitir** (API viva; esta reference fija el PATRÓN, no la firma canónica).

## Gate humano de escrituras — `canUseTool`

El bucle del Agent SDK acepta un callback `canUseTool(toolName, input)` que se invoca **antes** de
ejecutar cada herramienta. Es **asíncrono y bloqueante**: devuelve una promesa que el bucle espera.

- Toda herramienta **con efecto** (escritura ORM, comando en sandbox, envío externo) DEBE pasar por
  aquí. Lecturas puras pueden auto-aprobarse.
- El callback publica la propuesta a la UI de Odoo (por `bus_service`, ver `odoo-integration.md`) y
  **bloquea** hasta que el humano confirma o rechaza.
- **Timeout + rechazo por defecto**: si nadie responde en N segundos, resuelve como *denegado* para
  no dejar la conversación colgada.
- El humano ve: nombre de la tool, argumentos (con PII enmascarada), y el efecto estimado.

```
loop = agentSDK.run({
  tools,                       // ORM vía MCP + shell-en-sandbox
  canUseTool: async (name, input) => {
    if (isReadOnly(name)) return { allow: true, input }
    const decision = await odooConfirm(name, input)   // publica a UI, espera respuesta
    return decision.approved ? { allow: true, input } : { allow: false, reason: decision.reason }
  },
  resume: previousSessionId,   // reanuda conversación
})
```

## Streaming de progreso

El bucle emite eventos (texto parcial, tool_use, tool_result). Reenvíalos a la UI por
`bus.bus._sendone(target, "odoo_ai_agent.stream", payload)` y en el cliente
`bus_service.subscribe("odoo_ai_agent.stream", render)`. Para salida de alta frecuencia (una
terminal, logs de un comando) **agrupa** mensajes (debounce) para no saturar el bus.

## Resume de conversación

Persiste el `session_id`/estado del bucle (en un modelo Odoo, p.ej. `x_ai_agent.conversation`) para
poder reanudar (`resume`) tras recargar la página o entre turnos. No guardes secretos de tokens en
claro en ese modelo.

## MCP scoped a permisos

Las herramientas ORM del agente se sirven vía un server MCP construido con `common-skills:odoo-mcp-builder`
(deny-by-default, mask de PII, `orm_full_access` en server separado para escritura). El agente NUNCA
recibe acceso ORM más ancho que el del usuario autenticado (`su=False` → record rules).
