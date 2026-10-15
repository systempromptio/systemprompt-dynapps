# Orquestación de módulos custom (v2.0)

Desde la integración con `indaws_migration_kit`, la decisión por módulo custom se cristaliza en `migration.yaml` (FASE 2) y la ejecución vive en los **playbooks** específicos:

| Decisión | Playbook |
|---|---|
| Custom → Studio | `playbooks/modules/custom_to_studio.md` |
| Custom → Core / Indaws Core | `playbooks/modules/custom_to_core.md` |
| Custom → STUB (preservar datos) | `playbooks/modules/custom_to_stub.md` |
| Custom → portar código v18 | `playbooks/modules/custom_port.md` |
| Report QWeb complejo → docx | `playbooks/modules/custom_report_to_docx.md` |

## Cuándo leer este fichero

Solo si se necesita el detalle histórico (pre-scan 9 greps, triage manual con AskUserQuestion).
**Versión pre-kit archivada** fuera del repo (`$SKILLS_ROOT/_archive/odoo-database-upgrader_legacy_*`). El detalle anterior ha sido migrado al motor `indaws_migration_kit` + playbooks en `references/playbooks/`.

## Skills externas delegadas

| Skill | Cuándo |
|---|---|
| `odoo-module-upgrader` | Port de código fuente v_origen → v_destino |
| `odoo-studio-migration` | Recrear funcionalidad en Studio (no-code) |
| `odoo-pilot` | Validar funcionalidad post-port via API |
| `odoo-code-auditor` | Auditar calidad post-port |

## Comando del kit relevante

```bash
# Step en migration.yaml.steps_custom que invoca la skill
- id: port_<modulo>
  type: indaws_migration_kit.helpers.delegate.OdooModuleUpgrader
  params:
    module: <modulo>
    from_version: "15.0"
    to_version: "18.0"
```
