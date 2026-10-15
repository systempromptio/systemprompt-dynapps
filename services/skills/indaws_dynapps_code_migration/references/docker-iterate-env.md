# Docker — Entorno local para el iterate loop

El iterate loop (`indaws-migrate iterate`) necesita poder hacer **restore limpio del dump
en cada ciclo** (restore → apply → `-u all` → parse). Para eso se usa un contenedor
PostgreSQL dedicado — separado del PostgreSQL local donde vive la BD de desarrollo.

---

## docker-compose.yml mínimo

Crear en el workdir del cliente (`migration_kit_<cliente>/docker-compose.yml`):

```yaml
version: "3.8"
services:
  db:
    image: postgres:15
    environment:
      POSTGRES_USER: odoo
      POSTGRES_PASSWORD: odoo
      POSTGRES_DB: postgres
    ports:
      - "5433:5432"    # puerto 5433 para no chocar con el PG local en 5432
    volumes:
      - pgdata:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  pgdata:
```

**Por qué puerto 5433:** el PostgreSQL local (dev) corre en 5432. El contenedor Docker
para el iterate usa 5433 — así ambos coexisten sin conflicto.

---

## Arrancar el entorno

```bash
cd migration_kit_<cliente>

# Arrancar solo el contenedor de BD
docker compose up -d db

# Verificar que está listo
docker compose exec db pg_isready -U odoo
# → localhost:5432 - accepting connections  (dentro del contenedor)

# Crear la BD de trabajo
docker compose exec db createdb -U odoo <cliente>_migrate
```

---

## Configurar el kit para usar el contenedor

En `migration_kit_<cliente>/migration.yaml`, sección `context`:

```yaml
context:
  pg_host: localhost
  pg_port: 5433          # ← puerto del contenedor Docker
  pg_user: odoo
  pg_password: odoo
  pg_database: <cliente>_migrate
  dump_path: /ruta/absoluta/al/dump/original.dump
```

---

## Ciclo de iterate con Docker

```bash
# Comando completo (kit restaura el dump en cada ciclo)
indaws-migrate iterate \
  --dump $DUMP_PATH/original.dump \
  --workdir . \
  --strategy interactive

# El kit internamente hace en cada iteración:
# 1. dropdb <cliente>_migrate (en el contenedor)
# 2. createdb <cliente>_migrate
# 3. pg_restore → <cliente>_migrate
# 4. indaws-migrate run --pipeline <plan> --apply   (--pipeline es OBLIGATORIO)
# 5. odoo-bin -u all --stop-after-init
# 6. parse upgrade.log → classify errors
```

---

## Verificar que el iterate usa el contenedor y no el PG local

```bash
# Ver qué BD está usando el kit
indaws-migrate inspect --plan migration.yaml | grep pg_port
# Debe mostrar: pg_port: 5433

# Conectar al contenedor directamente para inspección
docker compose exec db psql -U odoo -d <cliente>_migrate -c "\dt" | head -20
```

---

## BD local de desarrollo vs contenedor iterate

| | PostgreSQL local (5432) | Contenedor Docker (5433) |
|---|---|---|
| **Propósito** | BD de desarrollo estable | Ciclos de restore limpio |
| **Persistencia** | Sí — la tocas con odoo-bin | No — se destruye en cada iterate |
| **odoo-bin** | Sí, en `odoo.cfg` apunta aquí | No — no se usa directamente |
| **indaws-migrate iterate** | No | Sí — apunta aquí |
| **Quién la gestiona** | Dev directamente | El kit automáticamente |

---

## Instalar Docker si no está

```bash
# Debian/Ubuntu
sudo apt update
sudo apt install -y docker.io docker-compose-plugin
sudo usermod -aG docker $USER
newgrp docker   # activar sin cerrar sesión

# Verificar
docker --version
docker compose version
```

---

## Parar el entorno al terminar la sesión

```bash
docker compose down        # para el contenedor, preserva el volumen
docker compose down -v     # para + borra el volumen (restore limpio en próxima sesión)
```

---

## Troubleshooting

| Síntoma | Causa | Fix |
|---|---|---|
| `port 5433 already in use` | Otro contenedor usa ese puerto | Cambiar a 5434 en docker-compose.yml + migration.yaml |
| `pg_restore: error: connection refused` | Contenedor no arrancado | `docker compose up -d db` |
| `FATAL: role "odoo" does not exist` | Imagen sin usuario odoo | Verificar `POSTGRES_USER: odoo` en docker-compose.yml |
| `iterate: dump not found` | `dump_path` incorrecto | Usar ruta absoluta en migration.yaml |
| `odoo-bin: could not connect` | odoo.cfg apunta al contenedor | odoo.cfg debe apuntar a localhost:5432 (PG local), no al contenedor |
