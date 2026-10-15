# Comandos del Mentor

Reference document with the full specification of each mentor command.

---

## /mentor-setup

Configuración inicial. Ejecuta el script de onboarding:

```bash
"$SCRIPT_DIR/setup_onboarding.sh" --email "<email>" --api-key "<api-key>" --role "<role>"
```

Después de que el script termine, crea la tarea programada:
```
create_scheduled_task(
    taskId="odoo-mentor",
    cronExpression="0 9 * * 1-5",
    description="Odoo Mentor - mentoring proactivo diario",
    prompt="Ejecuta el skill odoo-mentor. Sigue el workflow completo del SKILL.md."
)
```

Muestra el mensaje de bienvenida de `templates/onboarding-welcome.md`.

## /mentor-impersonate (solo admin)

Permite a un admin (Victor Peris) impersonar a cualquier empleado:

```bash
# Buscar empleado por nombre
"$PILOT_DIR/search_records.sh" hr.employee \
    '[[\"name\",\"ilike\",\"<nombre>\"]]' \
    '["id","name","user_id","department_id","job_title"]' 5

# Configurar impersonación
export MENTOR_IMPERSONATE_UID=<employee_user_id>
export MENTOR_IMPERSONATE_ROLE=<role>  # pm|developer|tl|sales|sales_manager
```

Opciones:
- `/mentor-impersonate María García` → buscar y configurar
- `/mentor-impersonate --role sales` → solo cambiar rol
- `/mentor-impersonate --off` → desactivar impersonación

## /mentor-uninstall

Desinstala Odoo Mentor del usuario actual. Ejecuta los siguientes pasos:

1. **Eliminar tarea programada**:
   ```
   Busca y elimina la tarea programada 'odoo-mentor' si existe.
   ```

2. **Eliminar credenciales**:
   ```bash
   rm -f "${DYNAPPS_HOME:-$HOME/.dynapps}/credentials/odoo-corporate.env" ~/.claude/.env.odoo
   ```

3. **Eliminar estado y memoria**:
   ```bash
   rm -rf "${DYNAPPS_HOME:-$HOME/.dynapps}/state/odoo-mentor/" ~/.claude/memory/odoo-mentor/
   ```

4. **Confirmar**: Muestra un mensaje de confirmación:
   ```
   ✅ Odoo Mentor desinstalado correctamente.
   - Credenciales eliminadas
   - Estado y memoria eliminados
   - Tarea programada eliminada
   Para reinstalar: /mentor-setup
   ```

**IMPORTANTE**: Pedir confirmación al usuario antes de ejecutar: "¿Estás seguro de que quieres desinstalar Odoo Mentor? Se eliminarán tus credenciales, estado y tarea programada."
