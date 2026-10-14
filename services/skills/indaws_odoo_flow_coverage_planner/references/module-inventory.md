# Inventario de modulos instalados

## 1. Obtener el inventario vivo

Conexion via `odoo-pilot` (ver `references/shared/odoo-connection.md`):

```bash
export ODOO_URL="https://instancia.odoo.com" ODOO_DB="..." ODOO_USER="admin" ODOO_KEY="..."
# Resolucion agnostica del agente — ver references/shared/skill-path-resolution.md
PILOT="${ODOO_PILOT_SCRIPTS:-$(ls -d ${DYNAPPS_SKILLS_ROOT:+"$DYNAPPS_SKILLS_ROOT"/*/skills/odoo-pilot/scripts} \
    "$HOME/.claude/plugins/cache"/*/*/*/skills/odoo-pilot/scripts \
    "$HOME/.config/opencode/skills/odoo-pilot/scripts" \
    "$HOME/.dyn-skills"/*/suites/*/skills/odoo-pilot/scripts 2>/dev/null | head -1)}"
eval "$(${PILOT}/auth.sh)"                 # autentica (auto-detecta JSON2 v19 / JSON-RPC <=18)
PILOT="$PILOT" scripts/list_installed_modules.sh > "$OUT/inventory.json"
```

`list_installed_modules.sh` lee `ir.module.module` con `state=installed`, campos
`name, shortdesc, author, license, application`.

## 2. Detectar la version (target_version)

```bash
"${PILOT}/detect_version.sh"     # imprime version; usar el major (ej "19.0") como target_version
```

## 3. Clasificar cada modulo

| Clase | Como detectar | Tratamiento |
|-------|---------------|-------------|
| **Estandar Odoo** | aparece en `catalog_map.json` con `testable=true` | reutiliza flujos de catalogo |
| **Tecnico/dependencia** | `catalog_map.json` `testable=false` (o `base/web/mail/...`) | excluir (sin flujo) |
| **Maestro** | `product`, `contacts`, `uom` | se prueba indirectamente via flujos transaccionales |
| **OCA / terceros** | `author` contiene "OCA"/"Odoo Community Association"; no esta en catalogo | `unknown` -> revisar a mano si es testeable |
| **Custom / Studio** | `studio_customization`, o `name` con prefijo cliente, o `author`=cliente; `license` LGPL/OEEL propietaria | `custom_gaps` -> generar flujo nuevo |

`build_coverage_matrix.py` hace esta clasificacion automaticamente a partir de
`catalog_map.json`. Los `unknown` (OCA/terceros/custom no catalogados) y `custom_gaps`
(Studio) son los que el Paso 2/3 debe resolver: o se mapean a una funcionalidad conocida
o se les genera un flujo a medida.

## 4. Custom: descubrir funcionalidad real

Para cada modulo `custom_gaps`/`unknown` que sea custom, lanzar `odoo-module-scanner`
(Agent) sobre su carpeta de codigo (si esta disponible localmente) para inventariar:
modelos nuevos, campos, metodos de accion (`action_*`, `button_*`), wizards, estados.
Eso alimenta la autoria de flujos en `flow-generation.md`.

Si NO hay codigo local (instancia remota), inferir funcionalidad via introspeccion API:
`ir.model` filtrado por `modules` del modulo, y `ir.model.fields` / `ir.actions.server`.
