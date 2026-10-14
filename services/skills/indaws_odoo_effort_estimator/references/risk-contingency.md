# Contingencia y Riesgo

Tabla unificada de contingencia por tamano de proyecto y tipo de trabajo.

**No se ajusta al aplicar factores IA.** Los porcentajes cubren la dispersion real de las desviaciones; el factor IA reduce el esfuerzo base, no el riesgo.

---

## Contingencia por Tamano de Proyecto

| Tamano | Criterio | Contingencia | Justificacion |
|--------|----------|-------------|---------------|
| **Pequeno** | < 10 modulos, 1 desarrollador | +10% | Riesgo bajo, equipo reducido |
| **Mediano** | 10-30 modulos, equipo medio | +18% | Riesgo moderado, coordinacion |
| **Grande** | > 30 modulos, equipo grande | +25% | Riesgo alto, dependencias complejas |

**Formula:**
```
Horas_Finales = ceil(Horas_Base × (1 + Contingencia))
```

---

## Contingencia por Tipo de Trabajo

### Implantacion Funcional
- **Estandar**: 10-15% de la bolsa total
- Cubre: imprevistos tecnicos, reuniones no planificadas, refinamiento requisitos, soporte extra go-live

### Migracion Tecnica (version upgrade)
- **Codigo**: Usar tabla por tamano de proyecto (10-25%)
- **Base de datos**: +15% sobre estimacion del Upgrade Service
- **Testing funcional**: +10% sobre horas de testing

### Desarrollo a Medida
- **MINOR/STANDARD**: +10% (modulos simples, riesgo controlado)
- **COMPLEX**: +20% (integraciones, logica compleja)
- **Con APIs externas**: +25% (dependencia de terceros)

---

## Ajustes por Calidad de Codigo (migraciones)

Estos ajustes se aplican sobre las horas base cuando se detectan problemas en el codigo existente:

| Hallazgo | Severidad | Ajuste | Categoria |
|---|---|---|---|
| SQL injection o `sudo()` excesivo | Critica | +30% Dev + QA | Seguridad |
| APIs deprecated (`@api.one`, `fields.function`) | Alta | +20% Dev | Deuda tecnica |
| Componentes OWL/JS complejos | Alta | +50% Dev (migraciones v16+) | Frontend |
| N+1 queries, `search([])` sin limite | Media | +15% Dev + QA | Performance |
| Codigo de mala calidad (sin patron claro) | Media | +10% QA | Calidad |
| Sin tests existentes | Media | +20% Testing | Calidad |

Se acumulan: un modulo con APIs deprecated y OWL complejo lleva +20% y +50%.

**Como se detectan** estos hallazgos: skill `odoo-database-analyzer` (patrones de busqueda) y
`odoo-module-scanner` (automatizado). Aqui vive **cuanto ajustan**, no como encontrarlos.

---

## Reglas de Aplicacion

1. **Contingencia se aplica DESPUES del calculo base**, no antes
2. **Redondear hacia arriba** (ceil) el resultado final
3. **Explicar al usuario** que porcentaje se aplica y por que
4. **Nunca ocultar contingencia** — es parte transparente del presupuesto
5. **Ajustar si hay informacion nueva** — la contingencia baja cuando se conoce mejor el alcance
