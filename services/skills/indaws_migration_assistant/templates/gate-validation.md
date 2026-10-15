# Template: Validación de Gate

> Usar cuando el usuario pregunta: "Gate X", "¿Puedo avanzar?", "¿Qué necesito para Gate X?"
> El LLM extrae la plantilla del gate específico de `references/gate-templates.md`
> y pre-rellena los campos si hay contexto del proyecto.

---

## Gate {gate_num}: {nombre_gate}

### Quién ejecuta / Quién aprueba
- **Ejecuta:** {quien_ejecuta}
- **Aprueba:** {quien_aprueba}

### Entregables por rol para este gate

| # | Entregable | Rol responsable | Estado |
|---|-----------|----------------|--------|
| 1 | {entregable_1} | {rol} | ⬜ |
| 2 | {entregable_2} | {rol} | ⬜ |
| ... | | | |

> Extraer entregables de `references/roles.md` para la fase correspondiente a este gate.

### Plantilla copy-paste

> Copiar la plantilla completa del Gate {gate_num} de `references/gate-templates.md`.
> Pre-rellenar: Proyecto, Cliente, Versión si se conocen del contexto.

{plantilla_gate_completa}

### Requisitos previos
- Gate {gate_anterior}: {estado_gate_anterior}
- {requisito_adicional}

### Qué desbloquea
- Avance a **Fase {fase_siguiente}: {nombre_fase_siguiente}**

---

## Reglas de formato

- Extraer la plantilla copy-paste COMPLETA de `references/gate-templates.md` para el gate solicitado
- Pre-rellenar Proyecto, Cliente, Versión si el usuario ya proporcionó contexto
- Incluir los entregables por rol de `references/roles.md` para la fase de ese gate
- Si el usuario tiene un rol identificado, destacar SUS entregables específicos
- Estado ⬜ por defecto; si hay datos del proyecto vía API, actualizar
