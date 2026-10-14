# Golden Example — Traspaso a Micaela

<!-- referenced from SKILL.md References table y desde templates/ficha_traspaso_micaela.md -->

**Ancla de regresión: antes de modificar el formato del correo de traspaso, confirma que
sigue esta estructura. Si cambia a propósito, actualiza este ejemplo.**

Caso real de referencia (TFT Food / Abuelo Bread), aportado por consultoría. Úsalo como
referencia de FORMATO y TONO, no copies literalmente contenido de otro cliente.

**Invariantes de formato (no negociables):**
- Saludo corto ("Hola {Nombre},") + una línea de contexto ("Te paso la ficha de {Cliente}").
- Bloque "Datos de cliente" en **tabla** de 2 columnas (campo | valor), NO en lista.
- El resto de bloques ("Usuarios y herramientas actuales", "1. Información administrativa
  y técnica", "2. Fases de implementación", "3. Notas adicionales") en **prosa/bullets**,
  NO en tabla — mezclar formatos rompe el patrón.
- Los encabezados numerados (1, 2, 3) se mantienen tal cual, calcan los bloques de
  `templates/ficha_traspaso_micaela.md`.
- Cierre corto: invitación a ampliar + despedida ("Saludos!!"). Sin firma larga.
- **CERO detalle técnico de GAPs**: nada de "Ref: sección 2.X", nada de las 5 columnas del
  análisis (Requisito/Funcionalidad estándar/GAP/Solución/Justificación). Todo va resumido
  a nivel de decisión (qué módulo, qué integración, por qué importa al negocio).

---

## Ejemplo (formato y tono de referencia)

Hola Mica,

Te paso la ficha de TFT

Datos de cliente

| | |
|---|---|
| Cliente | TFT Food, S.L. — marca comercial Abuelo Bread · CIF B72328545 |
| Alta en nuestra base de datos | 30/01/2023 (oportunidad de implantación abierta 07/04/2026) |
| Contacto / SPOC | Sara Montserrat Mateu — Jefa de Administración y Ventas |
| Correo | gestion@abuelobread.com |
| Teléfono | +34 617 32 18 45 |
| Sector | Fabricación alimentaria — productos panarios 100 % sin gluten y libres de alérgenos (galletas, picos, regañás, tostas, snacks). Certificaciones IFS, BRC, FSSC 22000, Sello Sin Gluten, Vegano |
| Ubicación | El Puerto de Santa María (Cádiz) — planta única |
| Tamaño | Pequeña empresa industrial, facturación aproximada 4 M€ |
| Empleados | 35–36 |

Interlocutores adicionales por área: Alejandro (producción), Ana (I+D), Cecilia y Tania
(calidad). La contabilidad la lleva una gestoría externa.

Usuarios y herramientas actuales

- Usuarios previstos en Odoo: 10, a revisar al alza según avance la implantación en planta.
- Herramientas que utilizan hoy:
  - A3 ERP (Wolters Kluwer) — contabilidad, gestionada por gestoría externa sobre servidor
    local con acceso VPN. Se mantiene por directriz de grupo.
  - WooCommerce (abuelobread.com) — venta online B2C. Se mantiene, con conector hacia Odoo.
  - SERES — plataforma EDI de entrada de pedidos de grandes superficies. Hoy los pedidos se
    transcriben a mano.
  - Skelo — RRHH. Fuera del alcance.
  - Excel y papel — recetas y escandallos, órdenes de fabricación, registros de calidad,
    valores nutricionales, inventario de equipos.
  - Sin ERP, sin MRP, sin CRM y sin trazabilidad digital en planta.
- Subvención: no consta. Pendiente de confirmar con el cliente.
- Intención de integrar toda la operativa en Odoo: sí, alta. Odoo pasa a ser el sistema
  central de compras, almacén, fabricación, ventas y facturación, y el emisor oficial de
  factura (responsable de VeriFactu). Excepciones acordadas: contabilidad en A3, RRHH en
  Skelo, y dos procesos que de momento siguen en Excel (no conformidades de calidad y
  cálculo nutricional).

1. Información administrativa y técnica

- Descripción del proyecto: implantación integral de Odoo 19 Enterprise en un fabricante
  alimentario que hoy no tiene ERP. El eje del proyecto es la trazabilidad completa por
  lote, requisito crítico para sus auditorías IFS/BRC. Incluye además el paso de servidor
  local + VPN a alojamiento cloud.
- Consultoría previa: análisis funcional completo ya entregado y validado por el cliente —
  sesión técnica de integración contable, visita presencial a fábrica y documento
  consolidado de 92 páginas cubriendo 9 bloques funcionales.
- Duración estimada: implantación de junio a noviembre de 2026, con go-live objetivo en
  noviembre de 2026.
- Customizaciones previstas (a alto nivel):
  - Integración EDI con SERES para entrada automática de pedidos de grandes superficies.
  - Integración contable Odoo → A3 ERP (exportación de asientos).
  - Etiquetado GS1 en impresoras Zebra vía IoT (EAN-13, DUN-14 y SSCC).
  - Integraciones con transportistas GLS y TXT.
  - Conector WooCommerce (contratado por el cliente).
  - PDF de orden de producción para auditoría IFS y generación de lote por día juliano.
  - Configuraciones avanzadas con Studio: certificaciones de proveedor, parámetros de
    proceso en planta, registro reglamentario de mermas.

2. Fases de implementación

- Módulos a implementar: Compras, Inventario, Ventas, Envíos, Fabricación (MRP),
  Facturación, Proyectos (I+D), Mantenimiento, más Código de barras, Taller, IoT,
  Información y Studio.
- Secuencia: primera actuación acordada, el etiquetado Zebra. El resto de módulos avanza
  en paralelo durante la configuración, con go-live conjunto en noviembre de 2026 y carga
  de inventario inicial (stock, lotes y caducidades) en el cutover.
- Fases con desarrollo: Ventas (EDI), Envíos (transportistas), Facturación (integración A3)
  y Fabricación (etiquetado e informes de auditoría). Compras, Inventario, I+D y
  Mantenimiento se resuelven con configuración y Studio.
- Riesgo de calendario a tener presente: el go-live coincide con la ampliación de la
  fábrica y con el pico de campaña de septiembre a diciembre.

3. Notas adicionales

- Previsión de crecimiento: alta y ya confirmada. En noviembre de 2026 entra en servicio
  una nueva línea continua de producción, tres hornos adicionales y una segunda
  envasadora. Cartera de unos 380 clientes y catálogo de unas 400 referencias, ambos en
  crecimiento. Es previsible que el número de usuarios suba tras el arranque, sobre todo
  en planta.
- Multianualidad: buena predisposición. Ya tiene contratado alojamiento con cuota anual y
  llega referido por otro cliente nuestro. No hay contrato de soporte cerrado todavía; es
  el siguiente paso natural.
- Upselling / cross-selling identificado: PLM (decisión aplazada 6-12 meses), portal B2B
  para grandes superficies y hostelería (solicitado expresamente por el cliente), CRM (no
  usan ninguno), Helpdesk, módulo de Calidad con registro estructurado de no conformidades,
  cálculo nutricional nativo, y una segunda fase de la integración contable (cobros, pagos
  y sincronización continua de maestros).
- Presupuesto: ajustado y muy vigilado por el cliente. Va con alcance cerrado y participa
  activamente recortando o sustituyendo partidas para no mover la cifra. Es un cliente que
  negocia el alcance, no un cliente con presupuesto abierto.
- Cliente conflictivo o consideración especial: no es conflictivo, al contrario, es
  colaborativo y asume tareas propias (depuración de maestros, alta de fichas
  nutricionales). Dos cosas a tener en cuenta:
  - La interlocutora conoce Odoo de antes por su trabajo anterior, así que no necesita
    demos básicas y sabe distinguir estándar de desarrollo.
  - Arrastra una experiencia previa fallida con EDI, por lo que ese bloque requiere
    gestión de expectativas.
- Otra información relevante: no tienen departamento de informática propio, todo el
  soporte debe poder darse en remoto. La contabilidad depende de un tercero (partner de
  A3/Wolters Kluwer), que condiciona plazos fuera de nuestro control. Y les aplica la
  nueva Ley de Desperdicio Alimentario de abril de 2026, que obliga a registrar la merma
  por destino: es un requisito sobrevenido después del alcance inicial y una necesidad
  funcional que puede repetirse en otros clientes del sector alimentario.

Cualquier cosa que necesites ampliar, me dices.

Saludos!!
