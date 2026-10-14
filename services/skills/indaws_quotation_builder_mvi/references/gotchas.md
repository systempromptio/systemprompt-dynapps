# Gotchas de quotation-builder-mvi

1. **Chrome headless no termina en macOS.** Escribe el PDF y se queda colgado; con `subprocess.run`
   y un timeout, la oferta «falla» con el PDF ya escrito. `render_pdf()` espera a que el fichero
   deje de crecer y cierra Chrome. Si tarda más de 90 s, algo va mal de verdad.
2. **La plantilla trae todas las líneas.** Crear el pedido con `sale_order_template_id` sin más
   deja las siete líneas de proceso aunque el cliente haya elegido una. Borra las no elegidas (servidor y
   mantenimiento se quedan siempre) y vuelve a leer
   `amount_untaxed` para compararlo con `price` antes de dar el pedido por bueno.
3. **Plataforma y líneas en el mismo pedido.** Funciona porque el pedido lleva `plan_id`
   (mensual): `MVI-SRV` y `MVI-MNT` son recurrentes y las líneas se facturan una vez. Sin `plan_id` la
   plataforma se facturaría una sola vez.
4. **Usuarios: manda el número, no la casilla.** «14 usuarios» con la casilla «10 o menos»
   marcada no es elegible. `check` y el formulario lo miran los dos.
5. **Nunca horas al cliente.** Las ~33 h estimadas y la comparativa con el value-based
   (`comparativa_interna`) son solo para quien cotiza. `lint_html()` bloquea «N h» y «N horas»;
   «1 hora funcional al mes» sí sale, porque es lo que el cliente compra.
6. **Sin franja de ajuste.** El precio es fijo (`ajuste_permitido_pct: 0`), y eso es lo que
   permite que lo cotice un SDR. Un descuento convierte la oferta en una negociación: pasa a
   comercial.
7. **Licencias de Odoo fuera.** Con Dyn Cloud el cliente necesita el plan Personalizado de Odoo,
   que contrata directamente con Odoo. No se meten en el pedido ni se calculan en la oferta.
8. **Veri*factu depende de Order-to-Cash y del certificado.** Se incluye con la línea O2C porque es
   ahí donde el cliente factura; con solo P2P o R2R el cliente factura en otro sistema, que es
   quien tiene que cumplir, y la oferta no lo promete (textos `*_sin_o2c` y `solo_con` del config).
   El certificado digital de la empresa se pide en los deberes del inicio: sin él no se envía
   nada a la AEAT y la iteración 3 no puede probarlo. Se usa el módulo estándar
   `l10n_es_edi_verifactu`; las variantes `indaws_*` (REAV, etc.) son desarrollos y quedan fuera.
9. **Producto nuevo sin cuenta de ingreso = ingreso a la 705000.** Las categorías `Servicios Odoo`
   y `Recurrentes` tienen la genérica `705000` como cuenta, y Dyn Brain prohíbe usarla. Pasó al
   crear los productos MVI: los cuatro nacieron en la 705000 hasta que se les puso la cuenta en la
   ficha. Todo producto MVI nuevo lleva `property_account_income_id` (ver `odoo_setup.md`).
10. **El metro ES el del value-based.** La red (seis líneas, estaciones, geometría, iconos y
    nombres de línea) se lee en tiempo de ejecución de `quotation-builder-value-based`
    (`shared/app_registry.json` → `flows`, y las constantes `_METRO_*` de `generate_proposal.py`,
    que se importa en 0,2 s sin efectos). No se copia: si el value-based añade una estación, el
    MVI la pinta sola. Del lado del MVI solo viven qué apps enciende cada línea MVI
    (`lineas.<id>.apps`), los nombres de apps en inglés, que el registro no traduce, y los textos.
    Los estados van por clases CSS en el `<svg>` (`on-o2c`…): la oferta los fija al generar y
    /qualify los cambia en vivo. Una línea del value-based se enciende en cuanto se contrata una
    línea MVI que pasa por ella (la de CRM enciende «Del presupuesto al cobro» aunque Ventas siga
    en gris), igual que allí. Lo que no es app de ninguna línea MVI (fabricación, calidad,
    mantenimiento, redes, automation, eventos, encuestas, selección, valoraciones, planificación,
    servicio de campo, Construcción) sale en discontinuo como «no disponible en MVI». Cada SVG
    lleva su `sid`: con dos mapas en la misma página, un degradado definido dentro de un bloque
    oculto no se pinta en el otro.
11. **Lo incrustado no se edita a mano.** El bloque `mvi-config` y el mapa de /qualify, y los
    mapas de las dos páginas del concepto, los escribe `scripts/sync_embeds.py` entre marcadores.
    Editarlos a mano funciona hasta el siguiente sync, que lo pisa.
12. **Siete líneas, pero de una a tres por MVI.** El SDR elige cuáles. `check` y /qualify cortan
    con cuatro o más (`limites.max_lineas`, `falla=lineas>3`): tres iteraciones no dan para más.
    Cada línea tiene su producto y su plantilla de texto; una línea nueva se añade en
    `config/mvi_offer.json` (con `apps` que existan en la red del value-based), su producto en
    Odoo y `sync_embeds.py`.
13. **El PDF no se sube apuntando al pedido.** `odoo_request_upload` con `model=sale.order` responde
    «not writable on this server». Se sube sin destino, se pasa en `attachment_ids` del
    `message_post` (la nota lo muestra) y después se escribe `res_model='sale.order'` y `res_id`
    en el `ir.attachment`, o no aparece entre los adjuntos del pedido. Medido con Airvolut
    (PR4274, 29-sep-2026).
14. **El pedido de Odoo sale en el idioma del contacto.** Odoo escribe el texto de cada línea al
    crearla, con el idioma del `res.partner`. Para un MVI en inglés: primero el idioma del contacto
    en `en_US` (si el cliente lo exige) y **después** las líneas. Los nueve productos MVI tienen
    traducción `en_US` de nombre y descripción (`update_field_translations`; el proxy no acepta
    `context` en `write`), sacada de `i18n.en.productos` del config. La nota de la plantilla es
    HTML con traducción por fragmentos y sigue solo en español: en un MVI en inglés se escribe la
    nota inglesa en el propio pedido. Medido con Airvolut (PR4274).
15. **El proyecto nace al confirmar, sin probar aún.** Los siete productos de línea tienen
    `service_tracking=project_only` con la plantilla «Plantilla · MVI» (id 1469, cinco tareas por
    fase): Odoo crea un proyecto por pedido y plantilla. No se ha confirmado todavía ningún pedido
    MVI, así que la creación del proyecto está sin verificar.
16. **Un «@Nombre» escrito en el cuerpo no notifica.** En `message_post` la mención solo avisa si
    se pasa `partner_ids`; como texto, se lee pero no llega.
17. **Una versión anterior del PDF no se borra, se renombra.** Borrar adjuntos es definitivo y no
    lo hace el skill: la versión vieja pasa a `OBSOLETO-vN-no-enviar-…` y la nota del pedido dice
    cuál la sustituye.
18. **La oferta sigue `utilities:dynapps-brand`, y el lint lo vigila.** Fondos claros solo blanco o
    gris niebla (`#EDEDED`), nunca lila, rosado, menta ni crema; un único momento oscuro, el cierre,
    con el logo blanco oficial, las credenciales y `www.dynapps.es` (nunca dynapps.eu); titular con la
    acción clave destacada en violeta («precio cerrado»), no el nombre del cliente; fases numeradas
    con cero delante (00 a 04); títulos de sección en SemiBold; etiqueta como *tag* violeta suave.
    `lint_html()` corta rayas, semirrayas, exclamaciones, dynapps.eu y fondos tintados. Los logos no
    se copian al skill: `brand_logo()` los resuelve como el value-based (`DYNAPPS_BRAND_ASSETS` o
    `dynapps-brand` de common-skills; si no, la copia oficial de `quotation-builder-value-based`) y
    los verifica por hash; si ninguno cuadra, la oferta sale sin logo y avisa, nunca con uno roto. Lo único de la guía que **no** se aplica son
    las etiquetas pequeñas en mayúsculas: la guía las permite, pero las ofertas del value-based
    prohíben cualquier mayúscula (CSS incluido) y el MVI sigue esa regla más estricta.

