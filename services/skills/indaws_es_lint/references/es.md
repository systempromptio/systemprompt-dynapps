<!-- es_lint: skip -->
<!-- Este documento se salta el linter a propósito: su tema SON las palabras
     prohibidas, y en las listas aparecen sin comillas. Es el caso que no-ai-slop
     describe como «una palabra usada como ejemplo, donde la frase es el sujeto y
     no la escritura». La exención es explícita, no un parche del linter. -->

# Referencia de idioma: español de España

Lee esto solo cuando edites un texto en español. Se usa con el `SKILL.md` de `no-ai-slop`, que tiene los principios compartidos y los conceptos de patrón.

Escrito para `utilities:no-ai-slop`, cuya tabla de idiomas marca el español como `_missing_` con dueño «needed: Dynapps ES». No es una traducción de `en.md`: el inglés y el español fallan por sitios distintos, y una lista traducida produce falsos positivos que enseñan a la gente a ignorar el linter.

La parte mecanizable de todo esto vive en `lint/brand_lint_es_web.json` y se comprueba con `lint/es_lint.py`. Aquí está lo que exige criterio.

---

## Lo que manda por encima de este fichero

La doctrina de la casa, en Knowledge, space 12:

- `como-trabajamos/que-publicamos-con-nuestro-nombre.md` fija la voz: **el sabio cercano, siempre «tú», nunca «usted»**, el diccionario de valor, y la ortografía completa desde la primera línea.
- `como-trabajamos/como-se-lleva-el-seo.md` fija algo que cambia el trabajo entero: **en la web no se traduce, se adapta al mercado local.** Traducir sin acoplar es lo que produjo el contenido duplicado que hay que arreglar.

Si esta referencia y esos documentos se contradicen, ganan ellos y hay que decirlo.

---

## Palabras que se quitan

**Prohibidas.** Aparecen casi solo en texto comercial de máquina.

potenciar, impulsar, maximizar, revolucionar, empoderar, robustecer, agilizar, dinamizar, optimizar (si no dices *qué* mejora), innovador, disruptivo, puntero, holístico, vanguardia, líder mundial, referente del sector, de clase mundial, llave en mano, solución 360, ecosistema integral, sinergia, transformación digital (salvo que sea literalmente el tipo de proyecto), la solución definitiva, tecnología de última generación, sin precedentes, un abanico de, una amplia gama de, llevar al siguiente nivel, desbloquear el potencial.

**Adverbios casi siempre vacíos.** Quítalos si no aportan; déjalos si llevan énfasis, duda, contraste o el habla del autor.

realmente, verdaderamente, simplemente, totalmente, absolutamente, completamente, enormemente, sumamente, altamente, indudablemente, evidentemente, obviamente.

**Frases casi siempre vacías.** Quítalas cuando retrasan el punto.

en un mundo cada vez más, en la era digital, en el panorama actual, hoy en día, a día de hoy, cabe destacar, cabe señalar, es importante destacar, conviene recordar, sin lugar a dudas, en definitiva, en resumen, en conclusión, en última instancia, como ya sabrás, no es ningún secreto, lo cierto es que, la realidad es que.

---

## Trampas estructurales del español

La sección que una traducción no puede producir, y donde está casi todo el valor.

### El gerundio de posterioridad

El equivalente español del `-ing` final del inglés («highlighting», «underscoring»), y **el tic número uno del español generado**. Un gerundio no puede expresar consecuencia posterior: es un error de gramática, no solo de estilo.

| No escribas | Escribe |
|---|---|
| permitiendo reducir costes | y así el coste baja un 12 % |
| logrando una mayor eficiencia | el cierre mensual pasa de seis días a dos |
| garantizando la trazabilidad | cada movimiento queda registrado con usuario y hora |
| posicionándose como referente | tiene 150 clientes en España |
| lo que se traduce en más control | el jefe de obra ve el margen de cada obra el mismo día |

> «Integramos el almacén con producción, permitiendo optimizar los tiempos de fabricación» pasa a «Integramos el almacén con producción. Las órdenes salen con el material ya reservado.»

### La perífrasis que entierra el verbo

El español administrativo esconde el verbo dentro de un sustantivo. Busca el verbo y úsalo.

| No escribas | Escribe |
|---|---|
| llevar a cabo la implantación | implantar |
| proceder a la revisión | revisar |
| hacer uso de | usar |
| dar cumplimiento a | cumplir |
| realizar la gestión de | gestionar |
| tener la capacidad de | poder |
| efectuar el pago | pagar |

### La pasiva refleja sin actor

«Se hará» esconde quién lo hace. En un documento de proyecto es así como desaparece la responsabilidad.

> «Se procederá a la configuración del módulo» pasa a «Configuramos el módulo la semana del 15.»
> «Los datos serán migrados» pasa a «Migramos los datos nosotros; tú validas el resultado.»

Nota que en español la pasiva refleja **sí es natural** en enunciados generales e impersonales («el IVA se declara trimestralmente»). Solo es un defecto cuando el lector necesita saber quién actúa.

### El residuo administrativo

| No escribas | Escribe |
|---|---|
| a fin de, con el objeto de, en aras de | para |
| el mismo, la misma (como pronombre) | repite el sustantivo, o «este» |
| dicho sistema, el citado proyecto | este sistema, el proyecto |
| en el marco de | en, dentro de |
| por la presente | quítalo |
| de cara a | para |

El «el mismo» pide cuidado: **como determinante es normal** y aparece en la web viva sin problema («hablan el mismo idioma», «la misma metodología»). Solo chirría como pronombre: «la misma se llevará a cabo».

### El español neutro, que no es el nuestro

Es la fuga más difícil de ver porque el texto es correcto. Llega siempre por el mismo camino: traducción automática desde inglés o adaptación desde otro país.

computadora → ordenador · celular → móvil · acá → aquí · rubro → sector · planilla → nómina u hoja de cálculo · tercerizar → externalizar, subcontratar · reporte → informe · cotización → presupuesto · chequear → comprobar · bodega → almacén · amigable → fácil de usar

### Calcos y falsos amigos

Solo los que tienen palabra española precisa. El jargon de dominio se queda tal cual.

| Calco | Español |
|---|---|
| en base a | según, con base en |
| de acuerdo a | de acuerdo con |
| eventualmente (por *eventually*) | finalmente, con el tiempo |
| librería (por *library*) | biblioteca |
| customizar | personalizar |
| soportar (por *to support*) | admitir, ser compatible con |
| asumir (por *to assume*) | suponer |
| consistente (por *consistent*) | coherente |
| aplicar a (por *to apply to*) | solicitar, presentarse a |
| remover (por *to remove*) | quitar, eliminar |

**No se traducen** y no son slop: Odoo, ERP, CRM, MRP, partner, Gold partner, sprint, backlog, go-live, módulo, FIT/GAP, stock. Traducirlos empeora el texto y, en la web, rompe palabras clave que el plan de SEO del grupo persigue.

### Las tildes se caen solas

La doctrina lo documenta como fallo real y progresivo: las tildes se pierden a medida que avanza un documento, porque el modelo imita la ortografía de lo que va leyendo. Se escribe con ortografía completa desde la primera línea, y si a mitad se detecta que están cayendo, **se reescribe el párrafo entero**. Al final ya no se ve.

El linter lo mide de dos formas: pares inequívocos y **deriva** de densidad de tildes entre el primer tercio y el último.

### Concordancia: los fallos que solo se ven escritos

Se oyen bien y se leen mal. Son de los pocos defectos que un hablante nativo comete a diario.

- **Contraste con número distinto.** «El problema no es el módulo, son los informes» chirría en papel. Reescribe con un solo verbo: «El problema no está en el módulo, sino en los informes.»
- **Porcentajes y colectivos.** «El 80 % de los pedidos se procesan» pide singular con el porcentaje como sujeto: «El 80 % de los pedidos se procesa solo.» Con «la mayoría de» ocurre igual.
- **Concordancia a distancia.** Cuanto más lejos queda el sujeto del verbo, más falla. Si la frase es larga, córtala antes de recolocarla.
- **«Haber» impersonal en plural.** «Habían tres incidencias» es incorrecto: «Había tres incidencias.»
- **Género del anglicismo.** Elige uno por término y mantenlo en toda la web: «la app» y «el ERP», no «el app» en una página y «la app» en otra.

### Longitud de frase: aquí el inglés se equivoca

El español tolera y pide frases más largas que el inglés o el neerlandés. La doctrina lo dice de la voz: «se toma el espacio que hace falta, párrafos largos y soluciones argumentadas cuando el asunto lo pide». **Editar español con la vara del inglés lo deja telegráfico y frío.** Corta una frase porque está enredada, no porque sea larga.

---

## Formas de los patrones en español

Los conceptos están en `SKILL.md`. Aquí las formas concretas, cada una con su reescritura.

**Contraste binario.** «No se trata de X, sino de Y» · «La pregunta no es X, es Y» · «No es solo X, es Y».
> «No se trata de instalar un software, sino de transformar tu organización» pasa a «El proyecto cambia cómo compráis, no solo qué programa usáis.»

Excepción: si corrige una suposición real del lector y esa corrección es el punto, se queda. Arregla solo la forma.

**Carraspeo inicial.** «Vamos al grano:» · «Seamos sinceros» · «Lo cierto es que» · «Te lo digo claro».

**Falso hallazgo.** «Lo que casi nadie te cuenta» · «Aquí es donde falla la mayoría» · «El error que cometen todos».
> «Lo que casi nadie te cuenta del ERP: la migración de datos» pasa a «La migración de datos es la mitad del esfuerzo del proyecto.»

**Revelación con dos puntos.** «El detalle que lo cambia todo: un solo stock».
> pasa a «Hay un solo stock, y eso es lo que evita el descuadre.»

**Grandilocuencia.** «juega un papel fundamental» · «supone un hito» · «marca un antes y un después» · «se consolida como» · «pone de manifiesto» · «de vital importancia».
> «Odoo juega un papel fundamental en la competitividad» pasa a «Con Odoo, el pedido llega a producción sin que nadie lo teclee otra vez.»

**Atribución vaga.** «los expertos coinciden» · «diversos estudios demuestran» · «está ampliamente reconocido» · «todo el mundo sabe» · «se estima que».
> Nombra la fuente o quita la afirmación. Nunca la inventes.

**Relleno especulativo.** «probablemente» · «todo apunta a que» · «cabe suponer» · «según la información disponible».
> «Probablemente su sistema actual no soporte facturación electrónica» pasa a «No sabemos qué versión tienes. Con la 12 o anterior, la facturación electrónica no entra.»

En un análisis o una propuesta esto pesa más que cualquier regla de estilo: la conjetura viaja como si fuera un hallazgo y acaba dentro del alcance.

**Coletillas apiladas.** «podría llegar a» · «en algunos casos podría ser posible» · «dicho esto» · «si bien es cierto que».
> Deja la que el significado necesita y arregla la exageración que venía a tapar.

**Objeción de paja.** «Que quede claro» · «No me malinterpretes» · «No estoy diciendo que» · «Ojo, no digo que».
> «No digo que la documentación no importe, pero la cuestión es si el consultor la encuentra» pasa a «La cuestión es si el consultor encuentra la documentación.»

Se queda la objeción que dice quién la puso, o la que el texto contesta de verdad después. Una afirmación directa («la API no es segura entre hilos») no es este patrón.

**Tríada forzada.** «personas, procesos y tecnología» · «más rápido, más simple y más escalable» · cualquier lista rellenada a tres por cadencia.
> «La migración aporta velocidad, claridad y control» pasa a «La migración deja el cierre mensual en dos días.»

**Rango falso.** «de principio a fin» · «de la A a la Z» · «desde la estrategia hasta la ejecución».
> «Te acompañamos desde la estrategia hasta la ejecución» pasa a «Hacemos el análisis, la construcción y el arranque.»

Excepción: un rango de verdad es un rango. «De Odoo 14 a Odoo 18» y «de lunes a viernes» se quedan.

**Rotación de sinónimos.** En web pesa doble: además de sonar a máquina, parte la señal de la palabra clave.
> En la web viva conviven «Partner de Odoo líder en España» y «el socio líder mundial». Elige una palabra, *partner*, y repítela.

**Verbo comodín.** El español infla el verbo donde «ser» y «tener» son más claros: «supone», «constituye», «se erige en», «se posiciona como», «cuenta con», «dispone de», «presenta».
> «La plataforma constituye un punto centralizado para la gestión de proveedores» pasa a «En la plataforma están los proveedores, sus tarifas y sus plazos de pago.»

**Enumeración negativa.** «Sin hojas de cálculo. Sin correos perdidos. Sin sorpresas.»
> Di lo que sí hay: «El albarán y la factura salen del mismo pedido.»

Excepción: una lista factual de lo que falta es un hallazgo, no un recurso retórico. «Sin control de versiones, sin entorno de pruebas, sin copias» en una auditoría se queda tal cual.

**Nombres soltados.** «empresas líderes confían en nosotros» · «más de 200 implantaciones de éxito» · «entre nuestros clientes se encuentran».
> «Empresas líderes del sector confían en Dynapps» pasa a «Nueve de nuestros clientes de fabricación usan este módulo en producción.»

Deja la referencia que trae un dato; quita la que solo pesa.

**Dicho formulario.** «X es el nuevo Y» · «los datos son el nuevo petróleo» · «el ADN de la compañía» · «la punta del iceberg» · «no es magia, es método».
> «La adopción es la moneda de la transformación» pasa a «El proyecto solo sale a cuenta cuando la gente deja de abrir el Excel viejo.»

**Sección de retos y perspectivas.** «Retos y oportunidades» · «Mirando al futuro» · «A pesar de estos retos» · «Aun así, quedan oportunidades».
> Quita la sección entera y quédate con los dos riesgos con nombre que enterraba.

**Metadiscurso interpretativo.** «Y esto es más importante de lo que parece» · «La clave está en» · «Como puedes ver» · «Es decir» redundante · «Dicho de otro modo».
> Se borran. Si el punto necesita apoyo, pon el dato, no el aviso de que el punto importa.

**Ritmo robótico.** Cuatro o cinco frases seguidas de la misma longitud y la misma forma, o párrafos calcados uno detrás de otro. Es lo más difícil de ver leyendo y lo más evidente leyendo en voz alta.
> Varía solo donde ayuda al argumento. No conviertas todos los párrafos en la misma pulcritud: eso es pasarse de edición.

**Alternativa falsa.** «Sería tentador» · «Uno podría pensar que» · «Lo obvio sería» · «Podrías creer que... pero».
> «Sería tentador rehacer el informe en SQL, pero eso salta las reglas de acceso. El informe usa read_group.» pasa a «El informe usa read_group, que respeta las reglas de acceso.»

Una opción rechazada puede ser real, y en una nota de diseño suele serlo. Varias rechazadas de pasada son residuo del borrador.

**Apertura complaciente.** «Buena pregunta» · «Tienes toda la razón» · «Excelente planteamiento» · «Totalmente de acuerdo».
> Se quita. Empieza por la respuesta.

**Slop de formato.** Titulares con emoji, negrita decorativa, viñetas donde el texto corrido lee mejor, un `H3` sobre dos líneas de contenido.
> El formato sigue al contenido, no lo decora. Una tabla que lleva datos no es slop de formato.

**Fragmentos apilados.** «Rápido. Sencillo. Eficaz.» · «Un solo stock. Una sola vista.»
> En un titular o una diapositiva es correcto y funciona. En un párrafo, fuera.

**Eco del titular.** Un `H2` seguido de una frase que solo lo repite.
> «## Rendimiento / El rendimiento es importante.» pasa a «## Rendimiento / Por encima de dos segundos de carga, uno de cada cuatro visitantes se va.»

**Cierre de efecto.** «Y eso lo cambia todo» · «El futuro ya está aquí» · «La diferencia está en los detalles» · «Porque al final, todo son personas».
> Bórralo, no lo mejores. Termina en la última frase concreta que ya tienes.

**Recapitulación final.** «En conclusión» · «En resumen» · «En definitiva» · «Para concluir».

**Residuo de chat.** «Espero que esto te ayude» · «Aquí tienes» · «¿Quieres que...?» · «¡Claro!» · «Por supuesto» · «Excelente pregunta» · «En este artículo veremos» · «A continuación te presentamos».

**Prosa de la versión anterior.** «esto sustituye al antiguo» · «a diferencia de antes» · «anteriormente se gestionaba con».
> Describe lo que hace ahora. La comparación se queda solo en notas de versión y guías de migración. Y hay una excepción viva: el aviso de rebranding («Indaws pasa a llamarse Dynapps») **tiene** que nombrar la marca vieja.

---

## Falsos positivos del español

`SKILL.md` trae la lista que vale en todos los idiomas. Esto es lo que solo aplica aquí, y lo que un editor con criterio inglés se llevaría por delante:

- **Los conectores.** El español necesita más conectores explícitos que el inglés para cohesionar. Un «sin embargo», un «además» o un «por otro lado» suelto **no es un hallazgo**. Un montón en el mismo párrafo, sí. El umbral del pack es tres.
- **La pasiva refleja impersonal.** «El IVA se declara trimestralmente» es español correcto y neutro, no responsabilidad escondida.
- **Las frases largas.** Ver arriba. La longitud sola no es un defecto en español.
- **Los signos de apertura.** `¿` y `¡` son ortografía obligatoria, no énfasis. No se «limpian» nunca.
- **«Solo» sin tilde.** La RAE no la pone. Marcarlo es un falso positivo.
- **«Le» y «les».** Son objeto indirecto de tercera persona («le pasamos el informe al cliente»), no necesariamente «usted».
- **«Vosotros».** Es la forma de España. Cambiarlo a «ustedes» es introducir una fuga de español neutro, no corregirla.
- **La cortesía que se queda.** «Un saludo», «Gracias por tu tiempo», «Quedamos a tu disposición» son cortesía real en el B2B español, no relleno. Lo que se va es el «usted» y el superlativo, no la educación.
- **Terminología precisa que parece folleto.** «Desarrollo a medida» es el término exacto de un desarrollo específico en Odoo. Lo que está prohibido es «a medida de tus necesidades». Se caza la frase, nunca el trozo.
- **Diminutivos y giros del autor.** Si el que escribe habla así, se queda.

Un patrón solo no prueba nada. Varios en el mismo párrafo son evidencia, y evidencia es lo que necesita un hallazgo.

---

## Registro por tipo de documento

| Documento | Se queda | Se va |
|---|---|---|
| Propuesta, respuesta a RFP, correo a cliente | Cortesía normal, apertura directa, despedida clara, el diccionario de valor | Superlativos, grandilocuencia, «nos complace comunicarle», todo el «usted» |
| Página de web, landing | «Tú», verbo en segunda persona, dato concreto, palabra clave del plan de SEO | Imperativo de usted en los botones, adjetivos de folleto, sinónimos rotados |
| Blog | Explicación larga y argumentada, opinión propia | Falso hallazgo, cierre de efecto, recapitulación final |
| Documento técnico, FUD, análisis | Terminología de Odoo y FIT/GAP intacta, riesgos y supuestos nombrados | Toda la cortesía de marco, la conjetura, el «como empresa» |
| Nota interna, correo interno | Taquigrafía, frases sin acabar, palabras claras: precio, presupuesto, mantenimiento, soporte | Todo lo ceremonial |
| Presentación | Fragmentos, ritmo hablado, una línea que cierre la sección | Superlativos, diapositivas que se vuelven párrafos |
| LinkedIn | La voz real del autor, incluida la opinión | Todos los patrones, sobre todo el falso hallazgo y el cierre de efecto |

**El tratamiento no se cambia nunca a mitad**, y en Dynapps no hay decisión que tomar: siempre «tú». Si el documento que editas está en «usted», eso no es el estilo del autor, es un defecto y se corrige.

**Ojo con el diccionario de valor.** Hacia el cliente se dice inversión, reto, acompañamiento técnico, evolución continua. **Dentro de casa se dice precio, problema, soporte y mantenimiento, y se dice a propósito**, porque un documento interno tiene que ser inequívoco. Aplicar el diccionario de valor a documentación interna es un error, no una mejora.

**Una diapositiva no es prosa.** El fragmento es correcto, y la coma donde iría un punto es ritmo hablado. Edita una presentación más suave que un documento: arregla lo que solo se ve escrito (concordancia, cláusulas apiladas, el mismo recurso repetido en diapositivas seguidas) y deja el ritmo en paz.

---

## Cobertura

Los treinta y cuatro conceptos de patrón del `SKILL.md` están cubiertos, cada uno con al menos una reescritura, salvo dos que no ocurren en español tal como los describe el inglés:

- **Straight quotes.** En español las comillas de la casa son los guillemets « », y el apóstrofo no es un problema tipográfico. La regla inglesa de endereza-comillas no aplica; lo que sí aplica es no romper un valor de campo de Odoo con comillas curvas.
- **False range** con «from X to Y» tiene su forma española propia («de principio a fin»), pero el rango legítimo es más frecuente en español técnico («de la versión 14 a la 18»), así que el umbral de sospecha es más alto.

Lo que aquí no está y hace falta seguir afinando, con el uso:

- **Formas del catalán, el gallego y el euskera** que se cuelan en la copia de mercado español. No hay material suficiente todavía.
- **El tuteo plural.** «Vosotros» frente a «ustedes» está resuelto, pero el paso de «tú» a «vosotros» dentro de una misma página (cliente como persona frente a cliente como empresa) no tiene regla escrita y hoy se decide a ojo.
