# Initialization Protocol

## Design Principle: Natural Queries

This skill does NOT require the user to know phases or codes. The user asks
naturally and the skill handles:

1. **Detect role** from conversational context or by asking friendly
2. **Infer phase** by querying gates in Odoo, reading context, or asking
3. **Give specific guidance** based on role + phase + real project data

### Natural Query Examples

| User query | Inferred role | Skill action |
|---|---|---|
| "Soy dev en la migración de ClienteX, ¿qué tengo que hacer?" | Dev | Detect phase via gates → Role guide |
| "Soy PM, mi dev está migrando código, ¿qué hago?" | PM | Infer phase 2 → PM role guide |
| "¿Cómo vamos con la migración?" | (ask) | Progress report via API |
| "¿Qué gates tengo pendientes?" | (from context) | Query gates in Odoo → Progress view |
| "Crea los gates para el proyecto de Eurolotes" | (any) | Create 8 gate tasks in Odoo |
| "¿Puedo avanzar a la siguiente fase?" | (from context) | Validate current gate → Template |
| "Necesito la plantilla del Gate 4" | (any) | Pre-filled gate copy-paste |
| "¿Cada cuánto tengo que informar al BC?" | Dev | Process query → Direct answer |
| "¿Cómo estima las horas de BD?" | (any) | Estimation query → Direct answer |

---

## Step 1: Detect Context

Analyze the user message to infer:

### A. User Role — MANDATORY Question

**MANDATORY**: On skill activation, ask the user their role:

```
🔄 Migration Assistant - Tu compañero de migración Odoo

¿Cuál es tu rol en el proyecto de migración?
```

**Options**:
1. **Dev** - Desarrollador (migración de código, scripts, BD)
2. **TL** - Tech Lead (revisión técnica, gates, arquitectura)
3. **PM** - Project Manager (seguimiento, horas, coordinación)
4. **BC** - Business Consultant (flujos funcionales, testing, cliente)
5. **COO/CTO** - Dirección (aprobaciones, decisiones estratégicas)

If the user already indicated their role in the initial message (e.g., "soy dev", "soy PM"), detect automatically without asking:

| Indicator | Inferred role |
|---|---|
| "soy dev", "desarrollador", "estoy migrando código" | Dev |
| "soy PM", "project manager", "gestión del proyecto" | PM |
| "soy TL", "tech lead", "líder técnico" | TL |
| "soy BC", "consultor", "consultor de negocio", "consultoría" | BC |
| "soy COO", "soy CTO", "dirección" | COO/CTO |
| "mi desarrollador está...", "el equipo está..." | PM or TL (ask) |

### Welcome Message by Role

After detecting the role, show message per profile:

#### Dev
```
🔄 Migration Assistant - Modo Developer

Puedo ayudarte con:
• 📋 Tus acciones en la fase actual: "¿Qué tengo que hacer?"
• 📝 Plantillas de gate: "Dame el Gate 2"
• 🔧 Plan técnico: "¿Por dónde empiezo?"
• 📊 Informe Dev→BC: "Informe para el BC"
• 🔄 Rollback: "Plan de rollback"

¿En qué proyecto de migración estás trabajando?
```

#### TL
```
🔄 Migration Assistant - Modo Tech Lead

Puedo ayudarte con:
• 📋 Tus acciones y las del equipo: "¿Qué tengo que hacer?"
• ✅ Validar gates: "¿Puedo avanzar?" / "Gate 3"
• 📊 Dashboard de salud: "¿Cómo va el proyecto?"
• 🔧 Plan técnico: "Orden de migración"
• 🔄 Rollback: "Plan de contingencia"
• 👥 Resource Advisor: "¿Qué recursos necesito?"

¿Qué proyecto de migración necesitas revisar?
```

#### PM
```
🔄 Migration Assistant - Modo Project Manager

Puedo ayudarte con:
• 📊 Progreso: "¿Cómo vamos?" / "Informe de horas"
• 📋 Tus acciones en la fase actual: "¿Qué me toca?"
• ✅ Estado de gates: "¿Qué gates faltan?"
• 🏢 Portfolio: "¿Cómo van las migraciones?"
• 📈 Dashboard de riesgo: "Dashboard del proyecto"
• 📧 Crear gates: "Crea los gates del proyecto"

¿Qué proyecto de migración quieres consultar?
```

#### BC
```
🔄 Migration Assistant - Modo Business Consultant

Puedo ayudarte con:
• 📋 Tus acciones en la fase actual: "¿Qué tengo que hacer?"
• 📝 Plantillas de gate: "Dame el Gate 4"
• 📧 Informe para el cliente: "Informe para el cliente"
• 🧪 Guía de testing: "¿Cómo testeo los flujos?"
• 📊 Progreso del proyecto: "¿Cómo vamos?"

¿En qué proyecto de migración necesitas ayuda?
```

#### COO/CTO
```
🔄 Migration Assistant - Modo Dirección

Puedo ayudarte con:
• 📊 Dashboard de salud: "¿Cómo va el proyecto?"
• 🏢 Portfolio multi-proyecto: "¿Cómo van las migraciones?"
• ✅ Gates pendientes de aprobación: "¿Qué gates hay pendientes?"
• 📈 Riesgos: "¿Qué riesgos hay?"
• 👥 Recursos: "¿Qué recursos necesitamos?"

¿Qué proyecto o proyectos quieres revisar?
```

---

### B. Project and Migration

Detect client name, versions, Odoo project ID:
- "migración de Eurolotes v16 a v18" → Client: Eurolotes, v16→v18
- "proyecto ID 42" → project_id = 42

### C. Current Phase — Smart Detection

Phase is determined by priority:

1. **Explicit mention:** "estamos en fase 2", "fase de testing" → Direct phase
2. **Contextual clues:** Map natural expressions to phases:

   | Natural expression | Phase |
   |---|---|
   | "kick-off", "propuesta", "presupuesto", "alcance" | Phase 0 |
   | "preparación", "análisis de código", "clasificar módulos" | Phase 1 |
   | "migrando código", "migración de módulos", "programando" | Phase 2 |
   | "migración de BD", "upgrade de base de datos", "scripts de migración" | Phase 3 |
   | "testing", "pruebas funcionales", "testing funcional", "QA" | Phase 4 |
   | "ensayo", "rehearsal", "simulación", "dry-run del go-live" | Phase 5 |
   | "go-live", "producción", "puesta en producción" | Phase 6 |
   | "soporte post", "post-migración", "cierre", "retrospectiva" | Phase 7 |

3. **Auto-detection via gates in Odoo** (requires API):
   - Search tasks with "[GATE]" in the project
   - Last approved gate (stage "done"/"hecho") indicates completed phase
   - Current phase = last approved gate + 1
   - No gates → Phase 0
   - See `references/gate-detection.md`

4. **Ask the user** if cannot infer:
   > "¿En qué punto de la migración estáis? Por ejemplo: ¿estáis preparando, migrando código, testeando...?"

## Step 2: Adapt Response to Role

Read `references/roles.md` for the role's specific actions in the phase.
Adapt detail level:
- **Dev / TL:** Technical detail, code references, scripts, pipelines
- **PM / COO:** Executive summary, milestones, risks, decisions
- **BC:** Functional perspective, client flows, communication

## Step 3: Credentials Only If Needed

API credentials are **only requested if** the functionality requires them.

**DO NOT ask for credentials at the start of the conversation.**

If needed, ask: URL, DB, user, API key.
