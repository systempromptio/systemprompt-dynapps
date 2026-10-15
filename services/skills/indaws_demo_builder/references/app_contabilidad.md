# Contabilidad (Accounting) - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| account_accountant | Contabilidad completa | Siempre (si app seleccionada) |
| l10n_es | Localizacion espanola | Si empresa en Espana |
| l10n_es_edi_sii | SII (suministro inmediato de informacion) | Si facturacion electronica Espana |
| account_payment | Pagos | Siempre |
| account_bank_statement_import | Importacion extractos | Si conciliacion bancaria |
| account_asset | Activos fijos | Si gestion de amortizaciones |
| account_budget | Presupuestos contables | Si control presupuestario |

## Configuracion

### Ajustes Principales
- **Plan contable**: Instalar localizacion del pais del cliente (l10n_es para Espana)
- **Moneda**: EUR para Espana (activar multi-moneda si opera internacionalmente)
- **Periodos fiscales**: Configurar ano fiscal (enero-diciembre normalmente)
- **Impuestos**: Verificar IVA 21%, 10%, 4% (l10n_es los crea automaticamente)
- **Diarios**: Verificar diarios de ventas, compras, banco, caja
- **Posiciones fiscales**: Nacional, Intracomunitaria, Extracomunitaria

### Features Opcionales
- Conciliacion bancaria: Configurar al menos 1 cuenta bancaria
- Recordatorios de pago: Activar para cobros pendientes
- Analytic accounting: Si hay centros de coste
- Lock dates: Configurar para mostrar control de cierre

## Datos Maestros

### Cuentas Bancarias
- 1-2 cuentas bancarias de la empresa (con IBAN ficticio)
- Diario bancario asociado

### Terminos de Pago
- Contado (Immediate Payment)
- 30 dias (30 Days)
- 30/60/90 dias (si es habitual en el sector)
- Fin de mes + 30 dias

### Impuestos
- Verificar que existen IVA 21%, 10%, 4% (ventas y compras)
- Configurar retenciones IRPF si aplica (servicios profesionales)
- Posiciones fiscales mapeando impuestos correctamente

## Flujo de Negocio - Distribucion por Estado

### Facturas de Cliente (account.move, move_type=out_invoice)
| Estado | % | Descripcion |
|--------|---|-------------|
| Borrador | 20% | Pendientes de validar |
| Publicada | 40% | Emitidas, pendiente de cobro |
| Pagada parcialmente | 15% | Cobro parcial registrado |
| Pagada | 25% | Totalmente cobradas |

### Facturas de Proveedor (account.move, move_type=in_invoice)
| Estado | % | Descripcion |
|--------|---|-------------|
| Borrador | 25% | Recibidas, pendientes de registrar |
| Publicada | 35% | Registradas, pendiente de pago |
| Pagada | 40% | Pagadas al proveedor |

### Extractos Bancarios
- 1-2 extractos con movimientos para conciliar
- Al menos 3-5 lineas de extracto conciliadas
- 2-3 lineas pendientes de conciliar (para mostrar en demo)

## Tips para Demo Impactante
- Tener facturas con diferentes terminos de pago para mostrar vencimientos
- Crear 1 factura rectificativa (credit note) para mostrar el flujo
- Tener extracto bancario con 2-3 movimientos sin conciliar para demo en vivo
- Mostrar el informe de cuentas por cobrar/pagar con datos reales
- Si SII activo, mostrar el estado de envio de facturas
- Tener al menos 1 factura vencida para mostrar recordatorios de pago

## Comandos API (odoo-pilot)

```bash
# Crear factura de cliente
create_record.sh account.move '{"move_type": "out_invoice", "partner_id": PARTNER_ID, "invoice_line_ids": [[0, 0, {"product_id": PROD_ID, "quantity": 1, "price_unit": 100.0}]]}'

# Publicar factura
execute_method.sh account.move INVOICE_ID action_post

# Registrar pago
create_record.sh account.payment '{"payment_type": "inbound", "partner_type": "customer", "partner_id": PARTNER_ID, "amount": 121.0, "journal_id": BANK_JOURNAL_ID}'
```
