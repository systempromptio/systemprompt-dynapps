# De dónde sale el material

Cinco fuentes, en orden de lo que más rinde. Ninguna se inventa: si una no devuelve nada esa
semana, el hueco se queda sin llenar y se dice, en vez de rellenarlo con una idea genérica.

## 1 · El barrido de Odoo, para el bloque de producto

La base pública del scraper de partners y clientes de Odoo. Es lo que convierte tres piezas de
producto al mes en un dato defendible en vez de opinión.

**Sectores dominantes en España** — el ángulo de la pieza que siembra:

```sql
SELECT industry, COUNT(*) AS n
FROM odoo_customers
WHERE country_of_origin = 'ES' AND stop_date IS NULL AND industry IS NOT NULL
GROUP BY 1 ORDER BY n DESC LIMIT 12
```

**Altas por mes** — el pulso del mercado:

```sql
SELECT to_char(start_date,'YYYY-MM') AS mes, COUNT(*) AS nuevos
FROM odoo_customers
WHERE country_of_origin = 'ES' AND start_date >= '2026-01-01'
GROUP BY 1 ORDER BY 1
```

Tres cosas que hay que saber antes de leer una cifra de aquí:

- **El país va en ISO**: `'ES'`, no `'Spain'`. Filtrar por el nombre devuelve cero filas y
  parece que no hay datos.
- **La fila viva es `stop_date IS NULL`.** Sin ese filtro cuentas historia, no presente.
- **Un pico de altas no es crecimiento.** Marzo de 2026 marca 3.487 altas españolas y es un
  reescrapeo masivo. Cualquier tendencia que cruce ese mes es falsa, y el dato llega limpio,
  que es lo que lo hace peligroso.

Referencia de septiembre de 2026: **5.578 clientes Odoo vivos en España**, entre 217 y 434
altas al mes, comercio y distribución 1.028 y fabricación 845.

Para consultas más finas —competidores, clientes que cambian de partner, mapa por provincia—
está la skill `scout-odoo`, que no se duplica aquí.

## 2 · El inventario propio, para casos y producto

La skill `content-collection` guarda el catálogo empaquetado: casos de éxito, módulos `[CORE]`,
vídeos de YouTube y el sitemap completo para enlazado interno. **Se consulta antes de proponer
un caso**, porque un caso ya publicado no se vuelve a estrenar.

## 3 · El CRM, para saber de qué sector hablar

Los ganados recientes por sector dicen dónde estamos convenciendo, y eso es lo que debería
mandar en el ángulo del mes. **Solo lectura**, y **nunca** para decidir qué se publica: las
acciones de Marketing son independientes del CRM.

## 4 · El calendario, para los eventos

Los eventos con fecha salen del calendario de Outlook del equipo. Un evento entra en el
calendario editorial **con su fecha real**, no con una fecha estimada, porque las siete piezas
del bloque se cuelgan de ella.

## 5 · El podcast y los huecos de blog

El invitado del mes y la fecha de grabación fijan la semana del bloque de podcast. Los huecos
de blog se coordinan con lo que lleva la agencia externa para no canibalizar consultas — se
mira, no se le da instrucciones.

## Lo que nunca es una fuente

- **Una idea genérica de ERP** sin dato ni caso detrás.
- **Una tendencia sacada de un pico de datos** del barrido.
- **Un caso de éxito sin consentimiento de imagen** registrado.
- **Un lead o un nombre de cliente** sacado del CRM para publicarlo.
