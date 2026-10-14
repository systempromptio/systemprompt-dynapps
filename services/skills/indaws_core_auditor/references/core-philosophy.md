# Filosofia Indaws Core

Fuente de verdad: articulo de Conocimiento de Odoo **"📄 InDAWS Core" (ID 1933)**
en https://www.indaws.es, mas las Golden Rules de `odoo-classification-guide`.
Este documento empaqueta esa filosofia para auditar sin conexion. Si el articulo
1933 cambia, este fichero debe actualizarse en un PR.

## Que es Indaws Core

Modulos de Odoo que son **producto reutilizable para varios clientes**, no
desarrollo a medida de un cliente concreto. Una mejora entra en Core cuando:

- Mejora el sistema y **sirve a numerosos clientes**.
- El estandar de Odoo **no lo cubre**.
- Ejemplo canonico citado en Conocimiento: facturacion por albaranes.

No es Core: un parche para el flujo particular de un solo cliente, una
integracion con un sistema propietario de un unico cliente, o un retoque
cosmetico de un cliente.

## Principios (el "por que")

1. **El cliente se adapta al sistema, no el sistema al cliente.** Codigo que
   "moldea" Odoo para replicar el proceso idiosincratico de un cliente es deuda
   tecnica que no se podra reutilizar ni migrar. Un modulo Core ofrece una
   solucion configurable, no el flujo de la empresa X.

2. **Acercarse lo maximo al estandar (no reinventar lo existente).** Reimplementar
   algo que Odoo estandar o un Core ya hacen es el peor fallo de filosofia Core:
   duplicas trabajo que habria que mantener y migrar en cada cliente y version.
   Antes de escribir, el consultor explora alternativas siguiendo la escalera de
   `odoo-classification-guide`: estandar -> Core -> Studio -> custom. El objetivo
   no es "evitar codigo" sino no duplicar lo que ya existe; y cuando hay que
   escribir, que sea codigo gobernado.
   *(Matiz IA: el peso Studio↔custom se mueve —codigo barato + gobernado por skills vs Studio
   ingobernable—; esto NO cambia la filosofia Core. Doctrina en `odoo-classification-guide/
   references/ai-impact-studio-vs-custom.md`.)*

3. **Multi-cliente por diseno.** El mismo modulo se instala en muchos clientes,
   muchas bases de datos, a veces multi-compania. Cualquier valor que dependa
   del cliente (datos fiscales, cuentas, partners, emails, endpoints, textos
   legales) debe ser **configuracion**, nunca codigo. Hardcodear un cliente
   rompe la reutilizacion: es el incumplimiento mas grave.

4. **Empaquetado y nomenclatura.** Core se vende por **paquetes por sector** con
   **suscripcion anual** y releases cada 3/6 meses. Por eso los modulos siguen
   `indaws_*`, se agrupan en paquetes por area y tienen manifest coherente: son
   un catalogo, no piezas sueltas.

5. **Documentacion en el modulo.** Cada modulo Core debe llevar README y
   manifest (`summary`/`description`) utiles, orientados al "por que" de
   negocio. (La filosofia Core ademas exige una ficha del modulo en el
   Conocimiento de Odoo, pero eso es un proceso interno del equipo y queda
   FUERA del alcance de este auditor: no se audita ni se reporta.)

## Proceso oficial (contexto, no se audita en codigo)

1. Identificacion de la mejora durante el analisis de implantacion.
2. Clasificacion: la tarea se etiqueta "inDAWS Core".
3. Analisis: el equipo de Producto decide si entra.
4. Documentacion (consultor) + comunicacion (Marketing).

Todos los miembros de consultoria participan en Indaws Core.

## Golden Rules heredadas de la clasificacion

Aplican ANTES que cualquier otra valoracion (ver `odoo-classification-guide`):

- **GR1 Data & Integrations**: import/export/conectores → debe resolverse con
  `indaws_data_import` / `indaws_data_export`, no con codigo custom.
- **GR2 Dashboards & KPIs**: → Spreadsheets nativo de Odoo, no custom.
- **Core Supremacy**: si ya existe un modulo Core que lo cubre, es OBLIGATORIO
  usarlo; no reinventarlo en un modulo nuevo.

Un modulo "Core" que viola una Golden Rule es una contradiccion: es el sintoma
mas claro de que la filosofia anti-codigo no se aplico.

## Como se traduce a una auditoria

| Principio | Que busca el auditor | Severidad tipica |
|-----------|----------------------|------------------|
| Multi-cliente | Hardcoding de cliente, logica `if cliente==`, datos en `data/` noupdate, falta de parametrizacion | Critico |
| Anti-codigo | Reimplementacion de estandar, violacion de Golden Rules, ignorar Core existente | Critico/Alto |
| Empaquetado | Nombre no `indaws_*`, fuera de paquete, manifest incoherente, uso de `x_`/Studio | Medio |
| Documentacion | Sin README/manifest util en el modulo (ficha Conocimiento NO se audita) | Medio |
| Mantenibilidad | Delegado a odoo-code-auditor (migrabilidad, conventions, calidad, seguridad) | Variable |

El peso de cada bloque en el Core Compliance Score esta en
`compliance-checklist.md`.
