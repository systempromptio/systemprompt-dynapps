# Criterios ICP (Ideal Customer Profile) - Indaws

## Definicion ICP

El cliente ideal para un proyecto de implantacion de Odoo en Indaws cumple:

| Criterio | Umbral minimo | Peso en scoring |
|---|---|---|
| Empleados | >= 10 | 30% |
| Facturacion anual | >= 1.000.000 EUR | 30% |
| Sector empresarial | Fit alto/medio con Odoo | 20% |
| Revenue esperado proyecto | >= 15.000 EUR | 20% |

## Algoritmo de Scoring

### 1. Puntuacion por empleados (0-100)

| Rango | Puntos |
|---|---|
| 0-9 | 0 |
| 10-24 | 40 |
| 25-49 | 60 |
| 50-99 | 80 |
| 100-199 | 95 |
| 200+ | 100 |

### 2. Puntuacion por facturacion (0-100)

| Rango EUR | Puntos |
|---|---|
| 0 - 499.999 | 0 |
| 500.000 - 999.999 | 30 |
| 1.000.000 - 2.999.999 | 60 |
| 3.000.000 - 9.999.999 | 80 |
| 10.000.000+ | 100 |

### 3. Puntuacion por sector (0-100)

**Fit alto (100 pts)**: Fabricacion, Distribucion, Comercio mayorista, Servicios profesionales, Tecnologia, Ingenieria

**Fit medio (60 pts)**: Retail, Construccion, Alimentacion, Logistica, Educacion, Salud

**Fit bajo (30 pts)**: Hosteleria, Inmobiliaria, Agricultura, Otros

### 4. Puntuacion por revenue esperado (0-100)

| Rango EUR | Puntos |
|---|---|
| 0 - 4.999 | 10 |
| 5.000 - 14.999 | 30 |
| 15.000 - 29.999 | 50 |
| 30.000 - 59.999 | 75 |
| 60.000 - 99.999 | 90 |
| 100.000+ | 100 |

### Score final

```
Score = (emp_score * 0.30) + (turn_score * 0.30) + (sect_score * 0.20) + (rev_score * 0.20)
```

## Umbrales de accion

| Score | Accion |
|---|---|
| >= 60 | **CUALIFICADO**: Asignar comercial por provincia, avanzar a etapa "Cualificado" |
| 35 - 59 | **REVISION**: Crear actividad para revision manual del responsable |
| < 35 | **DESCARTADO**: Sin accion automatica (revision manual si se desea) |

## Regla ICP estricta

Un lead cumple ICP si **simultaneamente**:
- Empleados >= 10
- Facturacion >= 1.000.000 EUR

Un lead puede tener score alto sin cumplir ICP (ej: sector excelente + revenue alto pero empresa pequena). El campo `meets_icp` indica cumplimiento estricto.
