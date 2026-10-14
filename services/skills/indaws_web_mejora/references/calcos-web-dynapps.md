# Calcos confirmados en www.dynapps.es

Las páginas `/es/` son la versión inglesa pasada por un traductor. No es una
sospecha: los ejemplos de abajo están tomados del HTML en vivo el 17-sep-2026
con `bin/extraer.py`. El linter de tono no los caza —son palabras españolas
correctas, solo que no son la palabra de este negocio—, así que esta lista es
el complemento manual de `es-lint`.

| En la web hoy | Original inglés | Lo que se escribe | Matiz |
|---|---|---|---|
| Tren (`/es/services`, H3) | Train | **Formación** | «Train» es el verbo formar, no el vehículo. Es el calco más visible del sitio |
| Anfitrión (`/es/services`, H3) | Host | **Hosting** | El servicio se llama hosting; «alojamiento» solo si el contexto ya dice que es de servidores |
| Socio / Socio Oro / Socio Implementación | Partner / Gold Partner | **Partner** / **Odoo Gold Partner** | Es el nombre del programa de Odoo y la keyword real («odoo partner», 880/mes). «Socio» es el que tiene participaciones |
| Socio Gerente (`/es/about`) | Managing Partner | **Socio director** o el cargo real | Aquí «socio» sí es societario; lo que chirría es «Gerente» |
| Perspectivas sobre… (`/es/blog`) | Insights | **Guías**, **análisis**, **lo que aprendemos** | «Perspectivas» no es una categoría de contenido en español |
| Servicio de asistencia de Odoo | Helpdesk | **Odoo Helpdesk** (el producto) / **soporte** (el servicio) | El producto se llama Helpdesk y así se busca; el servicio que presta Dynapps es soporte |
| a la perfección | seamlessly | **sin fricción**, **sin pasos manuales**, o se quita | Tres apariciones en tres páginas: es la muletilla del traductor |
| Acerca de Odoo / Acerca de Dynapps | About | **Aplicaciones de Odoo** / **Quiénes somos** | «Acerca de» es el título de menú traducido, no el de una página que posiciona |
| Descubrimiento (fase) | Discovery | **Análisis inicial** o **primera conversación** | Como nombre de fase de proyecto, en español no se entiende solo |
| Implementación | Implementation | **Implantación** | es-lint ya lo marca como rotación de sinónimos: una sola palabra en todo el sitio |
| Alojamiento gestionado / gestionamos | managed | según el caso | Vigilar: «gestionado» se cuela en todo |

## Dobles sentidos: la palabra correcta en el dominio equivocado

Añadido el 17-sep-2026 tras `/es/cases/nobi`. No son calcos del traductor: son
palabras que **yo** elegí bien para consultoría y que en una página de fábrica y
logística leen como otra cosa. El riesgo sube cuando el cliente del caso fabrica o
envía: el mismo párrafo usa el vocabulario de los dos mundos.

| Inglés | En consultoría es | Pero en la página leía | Qué se escribe |
|---|---|---|---|
| Train | **Formación** (o entrenamiento) | Tren | **Formación**. Es el calco fundacional de esta lista, `/es/services` |
| Delivery (model, team) | La **entrega del proyecto** | Envío de paquetes, junto a «entregas», «almacén», «distribuidores» | Se rodea: «el equipo que puso Dynapps», «cómo se trabajó». En un caso de fabricación, **«modelo de entrega» no se usa** |
| Setup / the setup | La **configuración** de Odoo | «Montaje» = ensamblaje de producto, en el mismo párrafo que «cada paso del montaje» | **Configuración**. «Montaje» se reserva al producto físico |
| Breakage, to break | **Fallos**, roturas del software | «Averías» = máquina parada en la planta | **Fallos** |
| Version upgrade | **Actualización de versión** | «Subida de versión» suena a subir un fichero | **Actualización de versión** |
| Patching the software | **Parchear** | — | **Parchear** se queda: es el término de desarrollo y el contexto («apaño») ya lo marca como chapuza, no como parche de seguridad |
| Bug | **Fallo**, o **bug** en texto técnico | — | En web de cliente, **fallo**. «Bug» solo en documentación técnica |

La regla que sale de aquí: **antes de entregar, se relee el texto desde el negocio
del cliente del caso, no desde el nuestro.** Si el caso es de fabricación, logística
o retail, las palabras de proyecto (entrega, montaje, avería, implantación, arranque,
despliegue) hay que mirarlas una a una. El linter no caza ninguna: todas son
español correcto.

Términos de servicio que **no se inventan**: «managed support» se entregó como
**«soporte gestionado»**, que es lo estándar, pero `Alojamiento gestionado` ya está
arriba en la lista de vigilar. Antes de consolidarlo en más páginas hay que
confirmarlo contra el catálogo de servicios de Dynapps España.

## Cómo se usa

Al reescribir, se pasa esta lista **además** del linter. Y cuando aparezca un
calco nuevo, se añade aquí con su página y su fecha: es un inventario de un
sitio concreto, no una teoría de la traducción.

Si un calco se repite en muchas páginas y se puede cazar con una expresión
regular sin falsos positivos, su sitio es el pack de `es-lint`
(`lint/brand_lint_es_web.json`, lista `terminology` o `leak_denylist`), no este
fichero. Al tocar el pack hay que volver a pasar `bash lint/tests/run.sh`: la
calibración dice que la copia aprobada no puede generar bloqueantes.
