# Fix Classification & Creation Guidelines

## Clasificacion obligatoria de fixes

| Categoria | Descripcion | Ejemplo |
|-----------|-------------|---------|
| **GENERIC** | Bug en script de migracion de Odoo base/EE. Aplica a CUALQUIER BD. | FIX-001: `portal` asume `website_id` existe |
| **OCA-CONDITIONAL** | Datos/artefactos de un modulo OCA. Solo si ese modulo esta instalado. | FIX-004: menus de `sale_order_type` |
| **CUSTOM-CONDITIONAL** | Modulo custom del cliente. Nunca en el catalogo del skill. | — |

## Reglas para crear un nuevo fix

1. **Clasificar primero**: base/EE (GENERIC) vs OCA/custom (CONDITIONAL)
2. **Nunca aplicar incondicionalmente** un OCA-CONDITIONAL — envolver en `IF EXISTS (SELECT 1 FROM ir_module_module WHERE name = '<modulo>' AND state = 'installed')`
3. **Cabecera estandar obligatoria**:
   ```sql
   -- FIX-NNN: Titulo descriptivo
   -- Categoria: GENERIC | OCA-CONDITIONAL (<modulo>)
   -- Error   : mensaje de error exacto
   -- Script  : script de migracion que falla
   -- Causa   : por que falla
   -- Condicion: cuando aplica
   -- Fuente  : proyecto y version donde se descubrio
   ```
4. **Idempotente**: `IF NOT EXISTS` / `DO $$ BEGIN ... END $$` / `ADD COLUMN IF NOT EXISTS`
5. **Solo GENERIC y OCA** en `references/fixes/`. Custom van en `fixes/*.sql` del proyecto.
