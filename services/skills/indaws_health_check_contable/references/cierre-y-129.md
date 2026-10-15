# El cierre: lo habitual es no regularizar

**Supuesto por defecto: la empresa no hace asiento de regularización.** Algunas la hacen a mano,
unos años sí y otros no, y alguna tendrá un módulo de cierre. Los chequeos de cierre tienen que
dar bien en los tres casos sin que nadie les diga cuál es, y **no regularizar no es un hallazgo**.

## Por qué la 129 acumula saldo deudor

Sin regularización, el resultado de cada ejercicio se queda vivo en los grupos 6 y 7 y la 129
solo se mueve con la **aplicación del resultado**. Aplicarlo contra la 129 sin haberla abonado
antes deja saldo deudor que **no vuelve nunca a cero** y crece cada ejercicio con lo repartido.

El balance PYMES de Odoo lo compensa: su línea «V. Resultados de ejercicios anteriores» suma el
grupo 12 y el resultado de 6 y 7 anterior al ejercicio, y el balance cuadra. Dónde se ve el saldo
depende de dónde se mire:

- **Mayor, exportaciones por cuenta y API**: el acumulado.
- **Sumas y saldos de Odoo**: solo los movimientos del ejercicio. Odoo trata la 129 como
  `equity_unaffected` y a cada inicio de ejercicio pasa lo arrastrado a una línea virtual
  «Resultado acumulado» sin código. Antes de la aplicación la 129 sale a cero.

**No es un error de proceso: es una RECOMENDACIÓN**, con el saldo a la vista. El informe dice
cuánto arrastra la 129 en el mayor y en el sumas y saldos, el anexo da el saldo tras cada
aplicación, y se recomienda regularizar 6 y 7 al cierre de cada ejercicio. El argumento es
práctico: quien lea la 129 por su código (una gestoría con el sumas y saldos, una herramienta de
análisis) verá una pérdida que no existe, y cada año mayor.

Este diagnóstico se corrigió varias veces hasta **contrastarlo con el informe que el cliente ve
en Odoo**, no con el mayor. Esa es la prueba que hay que hacer con cualquier chequeo de saldos.

## Lo que sí es hallazgo: resultado sin aplicar (chequeo 81)

`pendiente = saldo de la 129 + resultado anterior vivo en 6 y 7`. Con regularización lo segundo
es cero; sin ella, lo primero es deudor y lo compensa. Suma cero = todo aplicado. Si lo pendiente
es el resultado del último ejercicio y no han pasado seis meses desde su cierre fiscal, la junta
está en plazo: se dice y no se señala.

## Tres reglas

- **La regularización se reconoce por lo que mueve**: 129 contra grupos 6 y 7. Una aplicación
  del resultado es 129 contra reservas, 121 o dividendos. El diario no sirve: en el de cierre
  viven juntas la regularización, el cierre del balance y la apertura.
- **Nada del cierre se mira dentro del asiento de cierre.** En una empresa que no lo hace, el
  chequeo se queda en NO APLICA y el hallazgo se pierde. El impuesto sobre beneficios se busca en
  la cuenta de resultados de cada ejercicio cerrado.
- **El uso «Cierre contable» no depende del diario de cierre**: se reconoce por amortización,
  variación de existencias y aplicación del resultado. Si dependiera del diario, una empresa que
  no cierra en Odoo perdería los chequeos de impuesto y de la 129, que son justo los que importan.

Validado sobre la misma empresa en dos bases (con y sin regularización): mismo diagnóstico. El 81
solo se ha ejecutado en la base sin regularización más cuatro escenarios simulados.
