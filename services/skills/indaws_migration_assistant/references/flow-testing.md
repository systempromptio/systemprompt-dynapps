# Protocolo de Testeo y Réplica de Flujos

> Fuente: Artículo 2602, Sección 14
> Para la ejecución real de tests de flujos, usar el skill `odoo-flow-tester`.
> Esta referencia documenta el protocolo y catálogo de flujos.

---

## Objetivo

Verificar que los flujos de negocio del cliente funcionan correctamente en la versión destino de Odoo. Se hace **replicando flujos reales** en el entorno de testing o pre-producción.

---

## Modalidad A — Testeo en Staging

**Cuándo:** Fases 3, 4 y 5 (cualquier entorno que NO sea producción).

**Características:**
- Se crean registros libremente (pedidos, facturas, movimientos...).
- No es necesario revertir nada — el entorno es desechable.
- Se usa el entorno de testing configurado en Fase 1.

**Protocolo:**
1. El BC describe el flujo a testear (ej: "Venta completa con envío parcial y factura parcial").
2. El tester (BC, Dev o herramienta automatizada) ejecuta el flujo paso a paso.
3. En cada paso, se verifica:
   - ¿El registro se crea correctamente?
   - ¿Los campos calculados son correctos?
   - ¿Las transiciones de estado funcionan?
   - ¿Los documentos relacionados se generan (albaranes, facturas, asientos)?
4. Se documenta el resultado: OK o KO con detalle de la incidencia.

**Formato de documentación:**

```
FLUJO: [Nombre del flujo]
ENTORNO: [URL del entorno de staging]
FECHA: [DD/MM/YYYY]
TESTER: [Nombre y rol]

Paso 1: [Acción]
  Resultado esperado: [Qué debería pasar]
  Resultado real:     [Qué pasó]
  Estado:             [OK / KO]
  Evidencia:          [ID del registro creado o captura]

Paso 2: [Acción]
  ...

RESULTADO GLOBAL: [OK / KO]
INCIDENCIAS: [Lista de problemas encontrados]
```

---

## Modalidad B — Testeo en Producción (con reversión)

**Cuándo:** Validaciones puntuales post Go-Live (Fase 6, monitorización 48h) o auditorías.

**Características:**
- Se trabaja sobre la base de datos de producción real.
- **OBLIGATORIO revertir** todos los registros creados para no dejar rastro.
- Solo se usa cuando es necesario verificar con datos reales y configuración real.

**Protocolo:**
1. **ANTES de empezar:** Identificar todos los modelos que se van a tocar.
2. Ejecutar el flujo con `execute_flow.js --mode production --api-key KEY` — esto:
   - Captura un snapshot del estado previo (saldos contables, stock)
   - Registra automaticamente los IDs de TODOS los registros creados (directos + cascada)
   - Genera JSON con `records_tracked` y `pre_state`
3. Documentar cada paso (OK/KO).
4. **AL TERMINAR:** Revertir con `revert_flow.py`:
   ```bash
   # Primero dry-run para preview
   python3 revert_flow.py --tracking-file resultado.json \
     --url URL --db DB --user USER --key KEY --version VER --dry-run

   # Despues de aprobar, ejecutar revert real
   python3 revert_flow.py --tracking-file resultado.json \
     --url URL --db DB --user USER --key KEY --version VER
   ```
   El script aplica estrategias de reversion por modelo:
   - `account.payment`: draft → cancel → delete
   - `account.move`: draft → cancel → delete
   - `sale.order` / `purchase.order`: cancel → delete
   - `stock.picking`: cancel → delete
   - Otros: delete directo
5. **VERIFICACIÓN FINAL:** El script compara automaticamente el `pre_state` con el estado actual y reporta diferencias.

**Reglas de seguridad para testeo en producción:**
- **NUNCA** testear en producción sin backup reciente verificado.
- **NUNCA** testear en producción sin el TL informado.
- **SIEMPRE** usar `--mode production` para activar tracking automatico.
- **SIEMPRE** ejecutar `--dry-run` antes del revert real.
- **SIEMPRE** verificar reversión completa (pre_state = estado actual).
- Si la reversión falla, **PARAR inmediatamente** y consultar al TL.

---

## Catálogo de Flujos Estándar

Flujos mínimos que se deben testear en toda migración. El BC puede añadir flujos específicos del cliente.

| # | Flujo | Modelos involucrados | Complejidad |
|---|-------|---------------------|-------------|
| 1 | Venta simple | sale.order → account.move → account.payment | Baja |
| 2 | Venta con envío | sale.order → stock.picking → account.move | Media |
| 3 | Venta con envío parcial + factura parcial | sale.order → stock.picking (parcial) → account.move (parcial) | Alta |
| 4 | Compra simple | purchase.order → account.move → account.payment | Baja |
| 5 | Compra con recepción | purchase.order → stock.picking → account.move | Media |
| 6 | Transferencia interna de inventario | stock.picking (internal) | Baja |
| 7 | Regla de reabastecimiento | stock.warehouse.orderpoint → purchase.order | Media |
| 8 | Fabricación simple | mrp.production → stock.move | Media |
| 9 | Fabricación con componentes + subproductos | mrp.production → stock.move (múltiples) | Alta |
| 10 | Asiento contable manual | account.move (entry) | Baja |
| 11 | Conciliación bancaria | account.bank.statement → account.bank.statement.line | Media |
| 12 | Nómina (si aplica) | hr.payslip → account.move | Media |
| 13 | Integración API externa | Depende del conector | Alta |
| 14 | Generación de informe PDF | ir.actions.report → QWeb | Baja |

> **Nota:** Para ejecutar estos flujos de forma automatizada, usar el skill `odoo-flow-tester`.
> Este catálogo es genérico y reutilizable fuera de migraciones (implementaciones, auditorías, formación).
