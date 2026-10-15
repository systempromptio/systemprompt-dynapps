## 1. Modelo de Secuencias en Odoo 18.0

### 1.1 Estructura de `ir.sequence`

El modelo `ir.sequence` es el motor de numeracion automatica de Odoo. Cada secuencia genera
codigos unicos (ej: `SO001`, `INV/2026/0001`) usando una combinacion de prefijo, sufijo,
padding y un contador interno.

| Campo | Tipo | Descripcion |
|-------|------|-------------|
| `name` | Char | Nombre descriptivo (ej: "Secuencia de Facturas") |
| `code` | Char | Codigo tecnico usado para lookup (ej: `account.move.sequence`) |
| `prefix` | Char | Prefijo del numero generado. Soporta `%(year)s`, `%(month)s`, `%(day)s`, `%(range_year)s`, etc. |
| `suffix` | Char | Sufijo del numero generado. Mismos placeholders que prefix. |
| `padding` | Integer | Longitud minima del contador (ej: padding=4 → `0001`) |
| `number_increment` | Integer | Incremento por cada numero generado (normalmente 1) |
| `number_next` | Integer | Proximo numero en la tabla `ir_sequence` (valor interno PostgreSQL) |
| `number_next_actual` | Integer | Campo computado que lee/escribe el valor real del proximo numero. **Este es el campo que se debe usar para restaurar.** |
| `use_date_range` | Boolean | Si True, usa `ir.sequence.date_range` para rangos de fecha |
| `implementation` | Selection | `standard` (secuencia PostgreSQL) o `no_gap` (row lock, sin huecos) |

### 1.2 Mecanismo interno

```
[Creacion de registro]
       |
       v
model._get_sequence() → ir.sequence.next_by_code('sale.order')
       |
       v
ir.sequence._next() → incrementa number_next → genera "SO024"
       |
       v
[Numero asignado al registro - IRREVERSIBLE sin intervencion]
```

**Importante:** La generacion del numero es atomica. Una vez que `_next()` incrementa el
contador, el numero se considera "consumido" independientemente de si la transaccion
de creacion del registro se completa o no (en `implementation='standard'`).

### 1.3 Secuencias con rango de fechas (`ir.sequence.date_range`)

Cuando `use_date_range = True`, Odoo crea registros en `ir.sequence.date_range` para
cada rango de fechas. Cada rango tiene su propio `number_next_actual`.

| Campo | Tipo | Descripcion |
|-------|------|-------------|
| `sequence_id` | Many2one → ir.sequence | Secuencia padre |
| `date_from` | Date | Inicio del rango |
| `date_to` | Date | Fin del rango |
| `number_next` | Integer | Proximo numero para este rango |
| `number_next_actual` | Integer | Campo computado - usar este para restaurar |

Las facturas y asientos contables en Odoo 18.0 usan secuencias con rango de fechas
(tipicamente por ano fiscal). Ejemplo: `INV/2026/0001` → prefijo `INV/%(range_year)s/`,
donde cada ano tiene su propio contador.

### 1.4 Implementacion `standard` vs `no_gap`

| Aspecto | `standard` | `no_gap` |
|---------|-----------|----------|
| Mecanismo | Secuencia PostgreSQL (`nextval()`) | Row lock + UPDATE |
| Huecos posibles | Si (transacciones rollback) | No (lock hasta commit) |
| Rendimiento | Alto (no bloquea) | Bajo (bloqueo exclusivo) |
| Restauracion | Mas compleja (ALTER SEQUENCE) | Mas simple (UPDATE row) |

En produccion, las secuencias fiscales suelen estar en modo `no_gap`.

---

