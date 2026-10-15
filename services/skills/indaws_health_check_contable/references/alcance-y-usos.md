# Alcance: estados, universo, guardas y usos de Odoo

## Los cuatro estados, y la recomendación

- **LIMPIO**: mirado, el enlace está.
- **HUECO**: mirado, el enlace falta. Es lo único que se vende.
- **NO APLICA**: la empresa no usa esa funcionalidad. **No baja la cobertura.**
- **NO MIRADO**: no construido, la consulta revienta, o depende de un criterio que la empresa
  tiene que fijar. **Sí baja la cobertura, y se dice.**
- **RECOMENDACIÓN**: mirado, no es un error, pero conviene cambiar la forma de trabajar. Cuenta
  como mirado, no se vende como hallazgo y va al final del apartado 03 con el dato que la
  justifica (caso tipo: el saldo de la 129 en empresas que no regularizan, ver `cierre-y-129.md`).

**Un chequeo que revienta nunca sale en verde.** `run.py` envuelve cada implementación en
try/except y convierte la excepción en NO MIRADO.

Cobertura declarada = `(LIMPIO + HUECO + RECOMENDACIÓN) / (total − NO APLICA)`, sobre los usos
activos.

## La regla del universo

Cada chequeo declara sobre cuántos registros se pronuncia (cuarto elemento de su retorno) y el
ejecutor convierte en NO MIRADO cualquier LIMPIO con universo vacío. Lo comprueba la máquina,
no la memoria de quien lo escribió.

Caso real: en una pasada había cuatro falsos verdes. Intracomunitarias se pronunciaba sobre 11
facturas de 46 (el resto, terceros sin NIF), residuos de conciliación miraba 50 apuntes de 737,
centros analíticos daba verde sobre cero centros y uno devolvía verde sin comprobar nada. **Un
falso verde es el único fallo de este informe que no tiene arreglo después.**

La mayoría de los chequeos todavía no declaran universo: con una base vacía salen verdes. Ver
`CHANGELOG.md`, pendientes.

**Verdes de presencia** («hay 14 dotaciones de amortización») confirman que algo existe, no que
esté bien. Son legítimos, pero decidir por chequeo hasta dónde llegan y no acumularlos en el
apartado de lo correcto.

## Guarda de tipo de empresa, antes de ejecutar nada

Solo sociedades mercantiles: `res.company.chart_template` en `es_pymes`, `es_full`,
`es_canary_pymes`, `es_canary_full` (en v17+ es una selección en `res.company`). Con ESFL
(`es_assec`, `es_canary_assoc`) o cooperativas (`es_coop_*`) **el motor para**: los ingresos no
están en el 70 y los ratios saldrían a cero sin que nada salte.

**La plantilla puede mentir.** Fundaciones y asociaciones montadas sobre `es_pymes` son
frecuentes: movimiento real en 72x (cuotas, afiliados) o 65x (ayudas monetarias) = se comporta
como ESFL, parar igual. La discrepancia va al informe interno, nunca al del cliente.

Financiación finalista en una mercantil (movimiento en 74x/13x): se ejecuta, con flag interno.

## Capa 0: para qué usa Odoo esta empresa

Lo que importa no es que use todo Odoo, sino que lo que usa lo use bien. Once usos, uno por fila
del catálogo: núcleo contable · facturación emitida · facturación recibida · cobros y pagos ·
tesorería y banco · financiación · existencias · nómina · fiscal declarativo · cierre contable ·
analítica y controlling.

Cada uso se decide **por volumen real de registros, no por módulo instalado** (`usos.py`):

- **INACTIVO**: ni un registro. Sus chequeos a NO APLICA de golpe; una línea en el informe.
- **PARCIAL**: rastro incompleto. **Eso es el hallazgo**: «tenéis la herramienta a medias», la
  distinción con más valor comercial del informe.
- **ACTIVO**: se ejecuta entero.

El propósito se enseña, no se supone: el motor lo escribe en una línea del apartado 02 («Odoo se
usa aquí para X») y se confirma con el consultor antes de seguir. Si la empresa lo corrige, se
corrige el alcance.

## Las capas de siempre (`aplicabilidad.py`)

- **Módulo instalado**: `ir.module.module.state == 'installed'`.
- **Instalado pero sin uso** es NO APLICA igual (p. ej. `account_batch_payment` con cero remesas).
  Y se dice: «tenéis la herramienta y pagáis a mano».
- **Varias compañías, multidivisa activa**: condiciones de datos, no de módulo.
- **«Obligación: se pregunta»**: NO MIRADO, nunca verde.

Un chequeo sin sujeto se declara NO APLICA con su motivo **en lenguaje de cliente**, nunca con el
nombre técnico del módulo.

## Ejercicio fiscal, nunca natural

Todo lo que se agrupa por año va por el ejercicio fiscal de la compañía (`fiscalyear_last_month`,
`fiscalyear_last_day`): inicio de ejercicio, ejercicios cerrados, resultado, impuesto, plazo de
la junta, deudas de nómina, meses de amortización. Etiquetas «2025» si es natural y «2024/25» si
no. Helpers en `carga.py`: `fin_ejercicio`, `ini_ejercicio`, `ejercicio`.

Excepciones, porque Hacienda las mide por calendario: el **347** (año natural) y los
**trimestres del IVA**. Están marcadas en el código.

## La analítica no es una regla plana (chequeo 99)

Desde v17 la obligatoriedad la declara Odoo en `account.analytic.applicability` (dominio ×
categoría de producto × prefijo de cuenta × compañía → opcional / obligatorio / no disponible):

- **Hay un plan obligatorio**: señalar solo las líneas bajo esa regla que no llegan al 100 % en
  ese plan.
- **Sin regla, uso < 5 %**: NO APLICA. **Sin regla, uso > 95 %**: LIMPIO con observación (es
  costumbre, y la costumbre se cae al cambiar la persona).
- **Sin regla, uso 5–95 %**: HUECO. El hallazgo es «el dato analítico depende de que cada usuario
  se acuerde», no «faltan cuentas analíticas».

Fuera del denominador: cuentas de balance, 63x/66x/68x salvo regla por prefijo, líneas de
impuesto, diario de cierre y asientos anteriores al alta de la regla (la validación no es
retroactiva: eso es histórico, no incumplimiento).
