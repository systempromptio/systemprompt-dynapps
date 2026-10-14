### Backup obligatorio antes de SQL

```bash
# Desde el servidor:
pg_dump -Fc -f backup_pre_cleanup_$(date +%Y%m%d_%H%M%S).dump nombre_base_datos

# Verificar que el backup es valido:
pg_restore -l backup_pre_cleanup_*.dump | head -20
```

---

