# PATH A — On-Premise

Infraestructura: servidor propio del cliente, acceso SSH directo a la máquina de base de datos.

## Prerequisitos

- Acceso SSH al servidor PostgreSQL del cliente (`${CUSTOMER}-prd-db-deltablue` u hostname equivalente)
- Usuario `odoo` con permisos en PostgreSQL
- `python3` disponible en el servidor
- `rsync` disponible localmente
- `pg_restore` / `createdb` disponibles localmente

## Pasos

### 1. Conectar al servidor y preparar directorio

```bash
ssh ${CUSTOMER}-prd-db-deltablue
sudo su odoo
mkdir -p ~/migration
cd ~/migration/
```

### 2. Ejecutar el Upgrade Service

```bash
# -x mantiene el dump en disco en vez de auto-restaurar
# Cambiar "test" por "production" solo para go-live real (requiere STOP 4)
# Cambiar 17.0 a la versión destino real
python3 <(curl -s https://upgrade.odoo.com/upgrade) test -d odoo -t 17.0 -x
```

> El proceso puede tardar horas para bases de datos grandes.
> Usar `screen` o `tmux` para procesos largos si la sesión SSH puede desconectarse.

### 3. Revisar logs de upgrade

```bash
cat ~/migration/upgrade.log
```

Verificar que no hay errores críticos antes de continuar.

### 4. Transferir dump a máquina local

```bash
# En la máquina local (no en el servidor)
read -p "Enter customer name: " CUSTOMER
rsync -azP -v ${CUSTOMER}-prd-db-deltablue:/var/lib/postgresql/migration/upgraded.dump .
```

### 5. Crear BD local y restaurar

```bash
createdb ${CUSTOMER}_upgraded_base -T template0
pg_restore -d ${CUSTOMER}_upgraded_base upgraded.dump
```

### 6. Post-restauración

Ver `post-restore.md` para los pasos siguientes (neutralización, credenciales admin, copia dev).
