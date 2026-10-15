# Origen, medio y campaña

Un lead sin etiquetar no está perdido: está **sin poder sumar**. No aparece en el rendimiento
de ninguna acción, así que la acción que lo trajo no se puede defender ni repetir.

Son tres campos y contestan tres preguntas distintas. La fuente es
`como-trabajamos/origen-medio-y-campana.md` en el space 12, y manda sobre esta página.

| Campo | Pregunta | Qué va dentro | De quién es |
| :--- | :--- | :--- | :--- |
| **Origen** `source_id` | ¿Quién nos manda al visitante? | la plataforma concreta: LinkedIn, Google, Odoo Partner Page | del grupo, lista cerrada |
| **Medio** `medium_id` | ¿Cómo llega? | el tipo de canal | del grupo, lista cerrada de nueve |
| **Campaña** `campaign_id` | ¿A qué acción nuestra responde? | la acción, con nombre y año | nuestra |

Los nueve medios: `CPC`, `Social`, `Email`, `Organic`, `Referral`, `Events`, `Direct`,
`Outbound`, `Other`.

## El error que se comete siempre

**LinkedIn es un origen. El medio de ese mismo lead es `Social`.** Poner «LinkedIn» en el
medio no da error y parece razonable, y lo que rompe es la agrupación: pierdes el tipo de canal
y ya no puedes preguntar cuánto trae todo lo social, que es justo para lo que el medio existe.

## La campaña: `ES-<año>-<nombre>`

Es el único de los tres que rellenamos nosotros, y el único que hoy funciona. Convención de
destino: **`[CC]-[Año]-[Nombre]`**, por ejemplo `ES-2026-Rebuild`. Las campañas nuevas se crean
ya así; las que existen con la vieja convención `MKT <CANAL> <AÑO>` se dejan hasta enero,
porque renombrar una campaña viva reescribe la etiqueta de los leads que ya cuelgan de ella.

**La campaña se crea por acción y por año, y se renueva en enero.** El año en el nombre es lo
que permite comparar un canal contra sí mismo.

## Lo que no se hace

- **Crear un origen o un medio nuevo.** No es nuestro catálogo, y desde el go-live de One Odoo
  —previsto para el 1 de octubre de 2026— estará además bloqueado a administrador de sistema.
- **Meter una campaña, o el nombre de una persona, en el origen.** Quién refirió va en el
  contacto o en el chatter.
- **Poner una plataforma en el medio** en lugar del tipo de canal.
- **Filtrar o agrupar por nombre en vez de por id.** Existen a la vez `Jose Vicente` y
  `Jose vicente` como registros distintos: un filtro por nombre no los junta y nada avisa.
- **Reutilizar la campaña del año anterior.**
- **Dejar el medio vacío.** Desde octubre bloquea la salida de la primera etapa.

## Sin UTM, el hueco no pasa

Un hueco cuya UTM no esté escrita y bien formada **no llega a `propuesto`**. Se queda
bloqueado con el motivo escrito. No es burocracia: una pieza sin medición no se puede defender
después, y lo que no se puede defender no se repite ni se corrige.
