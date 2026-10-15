# Ejemplo completo

```
TL: "Vamos a planificar la semana"

> Ejecuta: snapshot.py 28 2026-03-23 --with-tickets
> Presenta 3 tablas:
>   - 3 pendientes de la semana pasada
>   - 8 tareas a planificar (6 con sugerencia, 2 sin horas)
>   - Capacidad: Brenda 30h, Daniel 32h, Akash 0h...
>
> "Hay 2 tareas sin horas (17115, 17268). Cuantas horas les asigno?
>  Para los pendientes, alguno esta hecho?"

TL: "La de Akash hecha. La 17115 ponle 4h. La 17092 a Brenda. El resto acepto."

> Redibuja tablas con cambios
> Tabla resumen de acciones
> "Confirmas?"

TL: "Si"

> Ejecuta execute_replan.py (1 hecha, 2 replanificadas)
> Ejecuta write_tasks.py (17115 -> 4h)
> Ejecuta execute_plan.py (6 tareas asignadas)
> "Hecho: 6 planificadas OK, 0 errores. Brenda: 22h libres, Daniel: 32h libres..."
```
