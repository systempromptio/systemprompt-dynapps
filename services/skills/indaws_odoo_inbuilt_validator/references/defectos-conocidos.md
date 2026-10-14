# Defectos ya conocidos del vertical inbuilt — 22

**Su único propósito: que nadie los reporte dos veces.** Si lo que acabas de ver
está aquí, no es un hallazgo nuevo: se confirma, se cita por su número y se
apunta si el síntoma ha cambiado.

Orden **por gravedad**, el mismo del parte de defectos. El detalle —pasos de
reproducción, dónde parece estar y si viene del backport o ya estaba en la
versión anterior— vive en el parte, no aquí.

Salvo que se diga lo contrario, **ninguno se ha corregido** y **todos se midieron
con ficheros `.bc3` fabricados**, nunca con uno real de obra.

---

## Grupo 1 — Mueven dinero o impiden cobrar

| # | Caminos | Síntoma en una frase |
|---|---|---|
| 1 | A-01 (A-07 lo diagnostica) | El presupuesto de **tres niveles se infla**: los subtotales de las subpartidas se suman además del de su partida padre. Importar sin mediciones da el total correcto |
| 2 | F-23 | Desde el **tercer periodo**, `origin` + `certified_lines` emite al cliente una **factura negativa**: las positivas son solo las certificadas en el periodo, las negativas restan todo lo facturado antes |
| 3 | E-02 | Certificar una línea **sin mediciones** no llega a la cabecera: la línea muestra su importe, la cabecera se queda a cero y la certificación no se puede facturar |
| 4 | D-05 | El **coste real del material nunca llega** al proyecto ni a la tarea: la línea no se entera de su propio reaprovisionamiento y no hay apuntes analíticos |
| 5 | *(hallazgo, fuera del catálogo)* | **Duplicar un presupuesto** arrastra `project_id`: «Crear proyecto» sobre la copia actualiza el proyecto de la obra original y le duplica las tareas, en silencio |
| 6 | G-08 | La casilla **«Excluir línea de producto» AÑADE líneas** en vez de quitarlas (el compute acumula sin limpiar): el pedido de compra sale con líneas repetidas y se compra el doble |

## Grupo 2 — Rompen o vacían una función entera

| # | Caminos | Síntoma en una frase |
|---|---|---|
| 7 | B-03, B-04 | Un parser en `import_mode = 'cost'` **revienta la importación a nivel SQL**: el trabajo queda en `running` con el campo `error` vacío, el pedido a 0,00 y el pedido bloqueado para futuras importaciones |
| 8 | B-01, B-02, B-05, B-06 | **«Modo de importación» y «Efecto sobre el precio» no cambian nada** para los porcentajes que vienen en la descomposición del `.bc3`, que es el caso normal. Activar `apply_global` tampoco lo rescata |
| 9 | A-05 | Desmarcar **«Importar costes» no quita los costes: vacía el presupuesto** — quedan solo las secciones de capítulo, ninguna partida |
| 10 | *(hallazgo, fuera del catálogo)* | El **cron de importación FIEBDC-3 se apaga solo** tras su primera ejecución (no fija `numbercall`), y con `numbercall = 0` reactivarlo a mano no basta |

## Grupo 3 — Mensajes, presentación y riesgos silenciosos

| # | Caminos | Síntoma en una frase |
|---|---|---|
| 11 | F-16 | El mensaje *«This certification already has an invoice.»* es **inalcanzable**: siempre salta antes el guardián de borrador |
| 12 | B-09 | El mensaje de «falta la categoría de coste» **sale duplicado**, concatenado consigo mismo sin separador |
| 13 | F-24 | Un proyecto **sin cuenta analítica factura sin reparto** y nada avisa: la rentabilidad de la obra queda ciega. Emparejado: un producto de certificación **sin impuesto** emite facturas de obra sin IVA |
| 14 | F-06, F-07, F-20, F-21 | En los modos `origin` + `all_lines`, la **línea negativa cae dentro de la última sección**, que pasa a mostrar un subtotal negativo. No pasa en `certified_lines` |
| 15 | A-10 | Al importar sobre un presupuesto existente, la **línea previa queda intercalada** dentro de una partida descompuesta. No afecta a los importes |
| 16 | *(interfaz)* | **Plurales sin traducir** en las tarjetas de proyecto: «4 Tasks» junto a «0 Tareas» |
| 17 | *(interfaz)* | El **diálogo de mediciones se titula «Measurements»**, en inglés, y es una ventana central del flujo de obra |
| 18 | *(interfaz)* | **Formato de porcentajes inconsistente**: para la misma medición, una columna muestra `100%` y la contigua `1,00`. El valor se guarda como fracción y solo una lo formatea |
| 19 | *(interfaz)* | Las certificaciones creadas antes de un cambio de divisa **siguen mostrando el símbolo antiguo** en la lista |
| 20 | *(parser BC3)* | Un **`~N` con líneas de medición tumba la importación entera** (`Type must be M or A`). `~N` es lo normal en una certificación o una modificación de obra |
| 21 | *(parser BC3)* | El **aviso de error del `~D` muere al formatearse** (`replace() argument 2 must be str`): en vez de avisar y seguir, tumba la importación |
| 22 | *(parser BC3)* | Una **virgulilla dentro de un texto rompe el troceado** de registros: un `~C` citado en un texto se lee como registro nuevo |

---

## Dependientes de la versión de Odoo

Estos NO salen en todas las instancias. Si el objetivo es de otra versión, es
esperable que no reproduzcan: dilo así, no lo apuntes como «corregido».

### Generar tareas revienta en **Odoo 19** · `uom.category_id`

Afecta a **C-02, C-03, C-04, C-05, C-06** (todos los modos de tareas menos «no
crear»).

```
AttributeError: 'uom.uom' object has no attribute 'category_id'
  inbuilt_sale_project/wizards/inbuilt_project_create.py  ->  _convert_qty_company_hours
```

Odoo 19 rehizo las unidades de medida y quitó `uom.uom.category_id`. La línea que
compara la categoría de la unidad del coste con la de la compañía no se adaptó.

**Se dispara sólo si la partida tiene costes de categoría TIEMPO** (mano de
obra), porque el cálculo de horas asignadas es lo único que llega ahí. C-01 pasa
porque no genera tareas y no lo llama.

Para medir los árboles de tareas a pesar del fallo: quita del pedido los costes
cuya categoría tenga `is_time`, y **dilo en el informe**. Con eso el bloque C se
recorre completo y el defecto queda acotado a una causa concreta.

## Trampa de recorrido: E-01 parece imposible y no lo es

**Síntoma:** el icono de la calculadora abre el diálogo *Measurements* en **solo
lectura** —seleccionar filas cambia el sumatorio pero no certifica, y la celda de
cantidad certificada no acepta escritura—. Como además E-06 confirma que la línea
con mediciones **rechaza la edición manual**, parece que no hay forma de
certificar una línea con mediciones. **La hay.**

**Mecanismo:** es el mismo botón declarado en dos vistas, con contextos distintos:

```xml
<!-- vista formulario de la línea -->
context="{'readonly': state != 'draft'}"          <!-- editable si borrador -->

<!-- vista LISTA de líneas -->
context="{'readonly': context.get('readonly', True)}"   <!-- por defecto TRUE -->
```

Y `action_detail_view` de la certificación mete `readonly: False` en el contexto
cuando el estado es `draft`. O sea:

| Desde dónde se pulsa | Contexto | Resultado |
|---|---|---|
| lista de líneas del **formulario** de la certificación | nadie fija `readonly` → `True` | **solo lectura** |
| botón **«Detalle»** de la certificación en borrador | `readonly: False` | **editable** |

**Cómo recorrer E-01:** certificación en borrador → botón **«Detalle»** → icono de
la calculadora → escribir la cantidad en *Cant. certificada* → **Guardar**.

Comprobado: 8 de 8 unidades en la medición → la línea queda a 8 de 12 (66,67 %) e
importe 8 × precio unitario, y la cabecera lo recoge.

**Qué sí anotar:** que desde el formulario el diálogo sale bloqueado **sin decir
por qué**. No sale mensaje: la celda simplemente no responde. Es la clase de cosa
que un consultor reporta como «no se puede certificar por mediciones».

## Cómo usar esta lista en el paso 5

1. Antes de escribir un defecto nuevo, buscarlo aquí por camino.
2. Si está: **confírmalo citando su número** y anota solo lo que haya cambiado
   (síntoma distinto, importe distinto, versión distinta). No abras entrada nueva.
3. Si no está pero se le parece: dilo explícitamente —«se parece al 8, pero…»—
   para que quien lea decida si es el mismo.
4. **Los defectos ya conocidos van en una sección separada de los nuevos.** Un
   informe donde se mezclan no se puede priorizar.

## Lo que NO está en esta lista y tampoco es defecto

- Los **cinco caminos bloqueados** (A-08, G-04, G-06, G-07, G-10): limitación del
  dato de prueba o del alcance de módulos. Ver `references/catalogo-caminos.md`.
- **E-04** (certificar por encima del 100 % debe dejar) y **G-05** (margen ≥ 100 %
  no toca precios): comportamientos declarados.
- Todo lo que sea **el sistema diciéndote que no**:
  `references/mensajes-error.md`.
