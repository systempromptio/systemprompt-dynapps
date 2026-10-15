### Procedimiento de conexion

```bash
# 1. Conectar por SSH al servidor de Odoo
ssh usuario@servidor-odoo

# 2. Obtener credenciales de PostgreSQL
# Normalmente en /etc/odoo/odoo.conf o /etc/odoo-server.conf
grep db_ /etc/odoo/odoo.conf

# 3. Conectar a PostgreSQL
psql -U odoo -d nombre_base_datos

# 4. Verificar base de datos
SELECT current_database();

# 5. SIEMPRE iniciar transaccion
BEGIN;
-- ... operaciones ...
-- Verificar resultados ANTES de COMMIT
ROLLBACK;  -- Si algo falla
-- COMMIT;  -- Solo si todo es correcto
```

