# Informe de Auditoria Indaws Core

**Modulo:** [nombre tecnico]
**Ruta:** [path]
**Version Odoo:** [version]
**Contexto:** [pre-PR / candidato a Core / Core existente]
**Fecha:** [fecha]
**Nivel tecnico:** [Alto/Medio/Bajo]

---

## Veredicto

> **[✅ Apto como Indaws Core | 🟠 Apto con reservas | 🔴 No apto como Core]**
>
> **Core Compliance Score: [X]/100**

| Bloque | Peso | Puntos | Estado |
|--------|------|--------|--------|
| A. Multi-cliente | 35 | [x]/35 | [✅/🟠/🔴] |
| B. Anti-codigo / cercania al estandar | 25 | [x]/25 | [✅/🟠/🔴] |
| C. Naming y empaquetado | 15 | [x]/15 | [✅/🟠/🔴] |
| D. Documentacion | 10 | [x]/10 | [✅/🟠/🔴] |
| E. Calidad y migrabilidad (delegado) | 15 | [x]/15 | [✅/🟠/🔴] |

**Bloqueantes (🔴): [n]** — deben resolverse antes del PR.

---

## Resumen ejecutivo

[2-3 frases: estado general del modulo respecto a la filosofia Core, el
incumplimiento mas grave y el camino mas corto para que sea apto.]

---

## A. Multi-cliente por diseno  [x]/35

[Hardcoding, logica por cliente, datos en data/, parametrizacion, acoplamiento.]

| Sev | Hallazgo | Ubicacion | Por que rompe Core | Como arreglarlo | Origen |
|-----|----------|-----------|--------------------|-----------------|--------|
| 🔴 | [hallazgo] | [file:line] | [principio] | [fix] | (propio) |

## B. Anti-codigo / cercania al estandar  [x]/25

[Reimplementacion de estandar, Golden Rules, Core Supremacy.]

| Sev | Hallazgo | Ubicacion | Alternativa estandar/Core | Origen |
|-----|----------|-----------|---------------------------|--------|
| 🔴 | [hallazgo] | [file:line] | [estandar / modulo Core / Studio] | (classification-guide/code-auditor) |

## C. Naming y empaquetado  [x]/15

[Nombre indaws_*, paquete, manifest, ausencia de x_/Studio.]

## D. Documentacion  [x]/10

[README/manifest presente y util? summary/description coherentes?]

## E. Calidad y migrabilidad (delegado a odoo-code-auditor)  [x]/15

[Sintesis: Migrability, Conventions, Security, Performance, Code Quality.
Criticos de seguridad elevados a 🔴. Si no se ejecuto, indicarlo.]

---

## Plan de remediacion priorizado

### Bloqueante — antes del PR
1. [accion] — [esfuerzo estimado]

### Alto — esta iteracion
1. [accion] — [esfuerzo]

### Medio / mejora
1. [accion]

---

## Apendice — delegaciones ejecutadas

- odoo-module-scanner: [ok / no]
- odoo-classification-guide: [veredicto]
- odoo-code-auditor: [scores por categoria / no ejecutado]
