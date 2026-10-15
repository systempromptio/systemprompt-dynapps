# Sandbox — orquestar contenedores desde Odoo con seguridad

> Patrón del build **web_shell** (2026-07). Aplica cuando el agente (u otra feature de Odoo)
> ejecuta **código o comandos no confiables**: nada de eso corre en el proceso de Odoo, sino en
> contenedores efímeros aislados. Es **deployment/infra-security**, no ORM. Verifica los flags de
> docker/compose contra tu entorno antes de producción.

## Regla nº1: NUNCA `docker.sock` en Odoo

Montar `/var/run/docker.sock` en el contenedor de Odoo = un shell/RCE en Odoo se convierte en
**root del host** (docker corre como root). El privilegio docker vive en un **orquestador aparte**
detrás de **`docker-socket-proxy`** con allowlist de llamadas:

- `CONTAINERS=1`, `POST=1`, `IMAGES=1`, `NETWORKS=1` (lo que el orquestador necesita)
- `EXEC=0` (sin `docker exec` arbitrario), y todo lo demás en 0
- Odoo le habla al orquestador por su API HTTP, nunca al socket directamente.

## Jaula de egress

El contenedor sandbox no debe tener internet directo:

- Bridge dedicado con `com.docker.network.bridge.enable_ip_masquerade=false` → sin SNAT → sin
  salida directa a internet. **PERO** el `publish` (DNAT del puerto) **sí** funciona.
- ⚠ **No uses `internal: true`** para esto: rompe el `publish` del puerto (no podrás exponer el
  servicio del sandbox). La ausencia de masquerade es la palanca correcta, no `internal`.
- Todo el egress permitido pasa por un **proxy allowlist** (dominios concretos), no libre.

## Drop de privilegios en el contenedor

```yaml
cap_drop: [ALL]
cap_add:  [SETUID, SETGID]   # imprescindibles: sin ellas no puede bajar a non-root
```
Sin `SETUID`/`SETGID` el arranque que hace `su`/`setuid` a un usuario no-root falla con
"Operation not permitted" → crash-loop. Añade solo esas dos.

## Endpoints de servicio (`auth='none'`)

Webhooks/callbacks del orquestador hacia Odoo con `auth='none'`: SIEMPRE token single-use (TTL
corto) o clave compartida; nunca `sudo()` sin comprobar el token. En **v19** además marca
`readonly=False` si el endpoint escribe (ver `odoo-version-api-reference` v19-diff §26).

## Ciclo de vida

Contenedor **efímero por sesión**: se crea al abrir, se destruye al cerrar (o por TTL/inactividad).
Límites de CPU/mem/pids por contenedor. Nada de estado persistente compartido entre usuarios.
