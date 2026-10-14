# Frameworks de Decisión Técnica

## 1. Pirámide Anti-Código Indaws

> "Indaws no desarrolla, aporta valor." — Política Dependencia Cero: No OCA, no deps externas.

```
1. Odoo Estándar 100%  → Acciones automáticas, Chatter, Actividades, config nativa
2. Odoo Studio         → Campos, vistas, automatizaciones visuales
3. Indaws Core         → ¿Módulo propio que ya cubre este caso?
   3b. Core parcial    → Módulo existe pero no cubre → ¿Mejora horizontal? → Mejora Core
4. Desarrollo Custom   → SOLO si 1-3 no aplican (como módulo Core horizontal, nunca específico)
```

50% de desarrollos planificados resultan no críticos post-go-live.

### Core existe pero no cubre el caso (nivel 3b)

Cuando un módulo Core resuelve parte del problema pero falta funcionalidad:

| Pregunta | Sí → | No → |
|----------|------|------|
| ¿La mejora es útil para otros clientes? | Mejora Core | Custom del cliente |
| ¿Se puede extender sin romper lo existente? | Mejora Core | Evaluar refactor |
| ¿El cambio es retrocompatible? | Mejora Core | Versionar con cuidado |

**Quién paga**: el cliente que solicita la mejora asume el coste, aunque el código se incorpore al Core. El beneficio para Indaws es que no genera deuda técnica custom.

## 2. Standard vs Custom — Checklist

| Pregunta | Sí → |
|----------|------|
| ¿Existe en estándar v17/v18/v19? | Usar estándar |
| ¿Cliente puede adaptar proceso? | Usar estándar |
| ¿Solo cambio UI (campo, tab, vista)? | Studio |
| ¿Requiere Python/herencia modelos? | Evaluar Core/Custom |
| ¿Es core del negocio (ventaja competitiva)? | Justifica custom |
| ¿Coste dev+mant < beneficio? | Justifica custom |
| ¿Se necesita en >1 cliente? | Candidato a Core |

## 3. Studio vs Código

> **Matiz IA (política 2026):** el criterio dejó de ser "evita código, prefiere Studio". Con IA el
> coste de escribir/mantener/migrar el módulo se desploma, y lo **decisivo** es la **gobernanza**:
> Studio lo toca cualquiera sin gate de revisión (inconsistente, riesgo para datos), un **módulo
> gobernado** lo lleva 1 técnico + IA + pipeline de calidad (auditorías, tests, Core compliance).
> → Para **lógica no trivial** (validaciones, flujos, integridad de datos, integraciones): **módulo
> gobernado**, aunque "cupiese" en Studio. Studio queda para lo **trivial/bajo riesgo**. Core intacto.

**Studio cuando** (trivial y de bajo riesgo): un campo, una etiqueta, reorganizar una vista, una automatización simple de un paso, prototipos. Montar un módulo no compensa.

**Módulo gobernado cuando** (lógica no trivial): validaciones/constrains, onchange complejos, flujos, integridad de datos, herencia de métodos, QWeb reports, APIs/integraciones externas, JS/OWL, algo que necesite Git/tests/multi-entorno.

**Por qué Studio pierde en lo no trivial**: vive en BD (no Git), sin control de versiones real ni gate de revisión, difícil colaborar, puede sobreescribirse en updates, sin tests → inconsistente y arriesgado para los datos.

## 4. Política No OCA

> **Indaws: No se usan módulos OCA.** Si no existe en estándar/Studio/Core, se desarrolla internamente.

Motivos: dependencia externa no controlada, riesgo abandono, cadenas de deps, conflictos con Core.

**Excepción** (requiere Co-CTOs): evaluar riesgos, documentar con ADR, plan de salida.

## 5. Scorecard Build vs Configure (Ponderado)

| Factor | Peso | Pregunta |
|--------|------|----------|
| Diferenciador negocio | x3 | ¿Hace único al cliente? |
| TCO 5 años | x3 | ¿Presupuesto es restricción? |
| Riesgo regulatorio | x3 | ¿Datos sensibles? |
| Urgencia time-to-market | x2 | ¿<3 meses? |
| Complejidad integración | x2 | ¿5+ sistemas? |
| Capacidad interna | x2 | ¿Equipo cualificado? |
| Personalización | x1 | ¿Workflows únicos? |
| Escalabilidad | x1 | ¿Crecimiento exponencial? |

Score: **<25** → Configurar | **25-45** → Híbrido | **>45** → Desarrollar

## 6. One-Way vs Two-Way Door

**One-Way (analizar a fondo)**: cambio versión Odoo, migración BD, cambio hosting, eliminar módulos con datos, cambio modelo datos fundamental.

**Two-Way (decidir rápido)**: campo nuevo, probar módulo en staging, cambiar vista, ajustar automated action, modificar reporte.

Mayoría de decisiones son Two-Way. No paralizar al equipo con análisis excesivo en decisiones reversibles.
