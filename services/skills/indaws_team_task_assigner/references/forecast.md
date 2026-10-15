# Forecast Multi-Semana

Proyecta capacidad vs carga para las proximas 4 semanas.

```bash
# Forecast desde proximo lunes, 4 semanas
python3 scripts/forecast.py 28

# Forecast 6 semanas desde fecha especifica
python3 scripts/forecast.py 28 --weeks 6 --week-start 2026-03-23
```

Output:
- Tabla semanal: capacidad bruta, ausencias, capacidad neta, planificado, backlog, delta
- Detalle ausencias previstas
- Backlog por semana (tareas sin planificar)
- Releases previstos
- Recomendaciones (semanas sobrecargadas, baja ocupacion, backlog alto)

**Usar despues de la planificacion semanal** para validar que las proximas semanas
no tienen cuellos de botella.
