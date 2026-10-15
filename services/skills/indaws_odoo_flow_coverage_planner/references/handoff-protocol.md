# Contrato de handoff al odoo-flow-tester

Esta skill **planifica/genera**; el `odoo-flow-tester` **ejecuta**. El handoff ocurre en el Paso 4.

## Que se entrega

Directorio de salida (`output_dir`, default `./flow_coverage_output/`):

```
flow_coverage_output/
  inventory.json              # modulos instalados clasificados
  coverage_matrix.md          # matriz modulo->funcionalidad->canal->flujo->estado
  scope.md                    # documento de alcance/analisis (lo consume el flow-tester Paso 0/1)
  flows/                      # flujos CUSTOM canal IA (contrato real, ya validados)
    CHAIN_*.yaml
  catalog_refs.txt            # flow_id de CATALOGO a ejecutar (no se copian; el tester los tiene)
  human_visual_checklist.md   # validaciones VISUALES (NO van al flow-tester; las hace un humano)
```

**Solo los flujos canal IA van al flow-tester.** El `human_visual_checklist.md` NO se entrega al
flow-tester ni se pide a este que genere capturas para que la IA las interprete (eso es lo
token-pesado que evitamos): se entrega directamente a la persona que valida el aspecto.

## Como se invoca el flow-tester

Lanzar el skill `odoo-flow-tester` (via Agent o como slash `/odoo-flow-tester`) con un mensaje
que rellena su **Paso 0 — Recopilacion**:

> Documento de analisis: `<output_dir>/scope.md`
> Flujos custom a ejecutar: todos los `<output_dir>/flows/*.yaml`
> Flujos de catalogo a ejecutar: los listados en `<output_dir>/catalog_refs.txt`
> Modo: A (documentar) | B (verificar) | C (ejecutar+revertir)  -> segun config.json
> Conexion Odoo: misma que esta skill (ODOO_URL/DB/USER/KEY o config.json)
> Entorno: staging | production
> Version: <target_version auto-detectada>

## Notificacion OBLIGATORIA de validaciones manuales (SIEMPRE)

**Cada vez que se ejecuta el handoff planner -> tester, se DEBE notificar al usuario, de forma
visible, la lista de validaciones manuales (canal `human`) pendientes.** No es opcional ni se
difiere: el usuario tiene que saber, en el mismo momento del envio, que el flow-tester NO cubre lo
visual y que esos items quedan a su cargo.

Formato de la notificacion (incluir SIEMPRE en la respuesta del handoff). **Cada item DEBE llevar
Ruta + Como probar + Criterio** (no basta "que mirar"):

```
⚠️ VALIDACIONES MANUALES PENDIENTES (no las cubre el flow-tester — son visuales)
Total: <n> items · Estimado: ~<min> min · Detalle: <output_dir>/human_visual_checklist.md
  1. <titulo> [<modulo>]
     Ruta: <menu completo o URL exacta>
     Como probar: <pasos concretos>
     Criterio: <que debe verse para OK>
  2. ...
Estos items quedan a cargo del usuario; el flow-tester solo valida los flujos canal IA.
```

Reglas:
- Listar los items REALES de `human_visual_checklist.md` (no un texto generico).
- **Cada item con su Ruta (menu/URL) y Como probar (pasos).** Nunca omitirlos ni dejarlos vagos.
- Mostrar la notificacion tanto al LANZAR el tester (Paso 4) como al CONSOLIDAR resultados (Paso 5),
  donde ademas se marca cuales siguen sin marcar por el humano.
- Si no hay items `human` (raro), decirlo explicitamente ("0 validaciones manuales pendientes").

El flow-tester continua con su propio flujo:
- Paso 1 — mapea el `scope.md` a su catalogo + integra los flujos custom.
- Paso 1.5 — **STOP**: presenta su tabla de flujos y pide aprobacion (independiente del Paso 3.5
  de esta skill; aquel aprueba EJECUCION en datos reales).
- Paso 2 — `verify_data.py` (precondiciones/permisos/modulos).
- Paso 2.5 (modo C / production) — Safe Mode + Snapshot.
- Paso 3 — `execute_flow.py` con prefijo `[TEST-YYYYMMDD-HHMM]`.
- Paso 4 — revert (ORM -> ORM avanzado -> SQL).
- Paso 5 — informe de resultados.

## Recoger resultados (Paso 5 de esta skill)

El flow-tester produce un informe (HTML/MD) con PASS/FAIL/SKIP por flujo. Mapear cada resultado
de vuelta a la `coverage_matrix.md`:

- `flow_id` -> funcionalidad(es) que cubre -> estado + evidencia (errores, registros creados).
- Actualizar el % de cobertura efectiva (no solo planificada): funcionalidades con flujo PASS.
- Los FAIL alimentan: si es bug de codigo -> `odoo-code-auditor`; si es flujo mal definido ->
  corregir el YAML y reentregar.

## Modos recomendados por contexto

| Contexto | Modo | Por que |
|----------|------|---------|
| Solo plan/documentacion | A | No toca la BD; entrega matriz + flujos |
| Validacion post-migracion (BD test neutralizada) | C | Ejecuta y revierte; evidencia real |
| Produccion | C con Safe Mode (STOP) | Requiere snapshot + restauracion de integraciones |
| BD con pocos datos maestros | B primero | Detecta precondiciones que fallan antes de ejecutar |
