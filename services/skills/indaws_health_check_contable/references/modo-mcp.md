# Modo mcp: el Odoo del cliente por un conector MCP

El motor es Python y no puede llamar a un MCP; Claude sí. En modo mcp (`--modo mcp`) cada
consulta del motor se sirve desde `DIR/extract/<clave>.json`; la que falta se apunta en
`extract/pendientes.json`, el paso termina sin guardar nada, Claude la trae por MCP y se relanza.
El resto de la cadena es idéntico al modo api.

## Antes de empezar: guarda de volumen

Todo lo que se trae por MCP pasa por el contexto de Claude y hay que volver a escribirlo a disco.
Contar primero por MCP los apuntes de la compañía (`account.move.line`, `search_count` con
`[["company_id","=",<cia>]]`) y los asientos:

- **≤ 5.000 apuntes**: modo mcp viable.
- **> 5.000**: no. Decírselo al consultor y pasar a modo api (odoo-pilot). Una carga parcial no
  vale: el motor la daría por completa.

Comprobar también que el MCP apunta a la base del cliente que se quiere revisar (`whoami` o leer
`res.company`), y avisar si ejecuta como superusuario: sirve para leer, pero ve más que un usuario.

## El bucle

```bash
python3 $M/hc.py init --out DIR --cliente "…" --corte 31/08/2026 --modo mcp --odoo-nombre <base> [--cia N]
python3 $M/hc.py carga --out DIR          # termina con «MCP: faltan N consultas»
python3 $M/extract.py pendientes --out DIR
```

Cada línea de `pendientes` es `{"clave","model","method","args","kwargs"}`. Por cada una:

1. Ejecutarla por MCP **tal cual**, con el `execute_kw` genérico del conector (p. ej.
   `orm_execute_kw` con `model`, `method`, `args`, `kwargs`), para que la respuesta tenga la misma
   forma que en modo api (many2one como `[id, "nombre"]`). Si el conector pagina, pedir con
   `offset`/`limit` hasta agotar.
2. Escribir la respuesta en un fichero JSON (una página por fichero) y guardarla:
   `python3 $M/extract.py guardar <clave> p1.json [p2.json …] --out DIR`.
3. Si Odoo responde con error (modelo que no existe, campo que no existe):
   `python3 $M/extract.py error <clave> "<mensaje>" --out DIR`. El motor lo trata igual que en
   modo api: el chequeo afectado sale NO APLICA o NO MIRADO, nunca verde.

Relanzar el paso. Las consultas dependientes (terceros de los apuntes, etc.) aparecen en la
ronda siguiente: `carga` converge en tres rondas y `run` en dos o tres más, porque la
aplicabilidad también consulta Odoo. `extract.py estado --out DIR` resume cómo va.

Con `carga` y `run` completos, `puntos`, `informe`, `pdf` y `excel` ya no tocan Odoo.

## Gotchas

- **Nunca rellenar a mano una respuesta** ni guardar `[]` por ir más rápido: una respuesta vacía
  inventada es un verde falso con otra forma.
- **La clave depende de la consulta exacta** (modelo, método, argumentos). Si cambia el corte, la
  compañía o el motor, salen claves nuevas: es lo correcto, no reutilizar `extract/` de otro corte.
- `limit` alto (100.000 en apuntes) en `pendientes` no significa que haya que traerlos todos de
  golpe: se pagina, y `guardar` concatena las páginas.
