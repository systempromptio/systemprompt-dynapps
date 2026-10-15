# Desacople de IA — no atarse a Claude (base + extensión por proveedor)

> Patrón del build **web_shell/dynapps_ai_assistant** (2026-07). Claude es el
> oficial/recomendado, pero el diseño permite enchufar otras IAs (OpenAI, local
> Ollama/vLLM, futuras) sin reescribir el gate ni el transporte.

## Qué ya es agnóstico (no lo acoples)
- El **front** habla un protocolo WS propio (`prompt`/`delta`/`tool_request`/
  `confirm`/`result`) — no sabe de Claude.
- Las **herramientas** son MCP (estándar) — reutilizables por cualquier IA.
- El **gate humano** (`_gate_decision`) es la barrera de seguridad y es
  provider-agnóstica: sobrevive en TODOS los providers.

## Dos capas (clave)
1. **Sidecar** (Python del contenedor): paquete `providers/` con un **registry** +
   clase base `AgentProvider` + adaptador por IA. `make_provider(name)` cae al
   baseline (Claude) si el nombre no existe.
   - **Tier 1** (Claude): SDK agéntico nativo con MCP + hook de permisos → adaptador fino.
   - **Tier 2** (OpenAI/local): LLM "pelado" → un **loop genérico compartido** lista
     las tools MCP como function-calling y enruta CADA tool-call por el gate.
2. **Odoo** (patrón base+extensión, como `payment`/`payment_stripe`): módulo base
   AGNÓSTICO (`_selection_dai_ai_provider()` devuelve `[]`); cada proveedor = un
   **módulo-extensión** (`..._claude` auto_install, `..._openai`, `..._local`) que
   amplía el Selection (`super()+[...]`), añade su config y la aporta al handshake
   (`_provider_config(user, provider)`).

## Reglas
- El **loop del agente NO es un módulo Odoo** → vive en el sidecar. "Un módulo por
  IA" solo aplica a la parte Odoo (opción del selector + credenciales/config).
- **NO crees el módulo-extensión de una IA sin implementación real** → sería opción
  muerta. El seam habilita añadirlo cuando exista el backend.
- El sidecar mantiene un `DEFAULT_PROVIDER` garantizado en la imagen (baseline).

## Pendiente al meter un Tier-2 real
Credenciales **cifradas** (nunca un `Char` en claro), deps en la imagen, y
generalizar el metering (cada provider devuelve su usage) en vez de parsear los
transcripts de un provider concreto.
