# Argumentos de Venta para Migracion

## 1. El coste de quedarse: retirada de Odoo.sh + recargo del 25% (URGENCIA PRINCIPAL)

> Fuente de verdad de fechas y politica: `references/shared/odoo-lifecycle-licensing.md`.
> El resumen de abajo la refleja; ante cualquier duda o cambio, manda el SSOT.

### 1A. Deadline forzado: retirada de Odoo.sh
Cada version se ELIMINA de Odoo.sh en una fecha fija (deadline tecnico duro): v14 -> 31 oct 2026,
v15 -> 2027, v16 -> 2028, v17 -> 2029, v18 -> 2030, v19 -> 2031. Un cliente en Odoo.sh con version
antigua no elige si migra: la plataforma lo obliga. Palanca directa para traerlo a servidores Indaws/Dynapps
antes de esa fecha, con control del calendario en vez de una migracion a contrarreloj.

### 1B. Recargo del 25% en renovacion Enterprise
Politica oficial de Odoo (confirmada, ver SSOT): +25% sobre la suscripcion anual para versiones fuera de las 3 mas
recientes (a partir de ~3 anos 6 meses desde el lanzamiento). Se factura desde abril 2026. Corte del 4 de julio de
2025: quien renovo DESPUES paga el +25% si esta en version antigua; quien renovo ANTES (tipico multianual que acaba
p.ej. en 2028) no se ve afectado hasta que venza el contrato. En multianual el +25% se aplica al precio anualizado
cada ano hasta actualizar. Aplica a Odoo.sh y On-Premise; Odoo Online no. Confirmar la fecha de renovacion del
cliente antes de afirmar el importe. Detalle, casos peliagudos y arbol de decision en el SSOT.

### Calculo de Impacto (recargo 25%)
```
Coste anual de licencias x 0.25 = Sobrecoste anual evitable
```
Ejemplo: Cliente con 50 usuarios Enterprise a 27,90 EUR/usuario/mes (plan Personalizado, facturacion anual, precio de lista Espana):
- Coste anual: 50 x 27,90 x 12 = 16.740 EUR
- Sobrecoste 25%: 4.185 EUR/ano adicionales
- En 3 anos sin migrar: 12.555 EUR desperdiciados

### Argumentario
> "Quedarse tiene fecha y precio: su version sale de Odoo.sh en una fecha fija y, si renueva en version antigua,
> paga un 25% mas cada ano sin recibir nada a cambio. Migrar ahora convierte una obligacion a contrarreloj en una
> decision planificada y una inversion financiera inteligente."

---

## 2. Deuda Tecnica Compuesta

### El Efecto Bola de Nieve
Cada version que se salta:
- Las personalizaciones se alejan mas del estandar de Odoo
- Las APIs deprecadas se acumulan
- Los modulos OCA pueden no tener version compatible
- El coste de migracion crece EXPONENCIALMENTE, no linealmente

### Datos Clave
- Migrar 1 version: coste base X
- Migrar 2 versiones: coste ~2.5X (no 2X)
- Migrar 3+ versiones: coste 4-6X (efecto compuesto)

### Argumentario
> "Aplazar la migracion provoca lo que se conoce como deuda tecnica: cada dia que pasa, sus personalizaciones se alejan mas del estandar de Odoo, haciendo que cada futura actualizacion sea exponencialmente mas compleja y costosa."

---

## 3. Seguridad

### Riesgos de Versiones Antiguas
- Vulnerabilidades conocidas no parcheadas
- Framework web obsoleto (susceptible a XSS, CSRF)
- Bibliotecas Python/JS desactualizadas con CVEs conocidos
- Sin parches de seguridad del fabricante para versiones fuera de soporte estandar

### Argumentario
> "Su sistema ERP contiene la informacion mas critica de su negocio. Mantenerlo en una version antigua es como dejar la puerta de su empresa abierta de noche."

---

## 4. Productividad y Nuevas Funcionalidades

### Beneficios Directos de Migrar
- **Interfaz rediseñada**: Menos clics, flujos mas agiles
- **Mayor rendimiento**: Tiempos de respuesta mas rapidos
- **App movil mejorada**: Gestion desde cualquier lugar
- **Nuevas apps**: Knowledge, Planificacion, Spreadsheet avanzado
- **IA integrada**: Funcionalidades de inteligencia artificial en v18

### Argumentario
> "La migracion no es solo una actualizacion tecnologica. Es una inversion directa en la productividad de su equipo, con una interfaz mas rapida, nuevas herramientas y menos clics para las tareas del dia a dia."

---

## 5. Upsell: Indaws Core

### Propuesta de Valor
- Indaws Core es la capa de valor que "abraza" Odoo
- Soluciones probadas para problemas de negocio reales no cubiertos por el estandar
- **Mantenida y migrada por Indaws** - garantia de continuidad
- Reduce dependencia de codigo a medida costoso de mantener

### Modulos Core Destacados
- `indaws_storage`: Gestion de almacenamiento base
- `indaws_storage_s3`: Conector para Amazon S3 / Cloudflare R2
- `indaws_report_docx`: Motor de informes en formato PDF/Word

### Argumentario
> "La migracion es la oportunidad perfecta para limpiar, optimizar y alinear su sistema con las mejores practicas. Reemplazaremos codigo a medida por funcionalidad estandar o de nuestro Indaws Core, haciendo su sistema mas robusto y facil de mantener."

---

## 6. Retorno de Inversion

### Formula ROI
```
ROI = (Ahorro licencias 25% + Ahorro productividad + Reduccion costes futuros) / Inversion migracion
```

### Factores de Ahorro
1. **Sobrecoste de licencias evitado**: 25% anual recurrente
2. **Reduccion de deuda tecnica**: Proxima migracion sera mas barata
3. **Productividad**: Menos tiempo en tareas rutinarias
4. **Seguridad**: Evitar costes de incidentes de seguridad
5. **Soporte**: Acceso completo al soporte de Odoo y de Indaws

### Argumentario
> "Al mantener su plataforma alineada con el estandar, cada nueva funcionalidad que Odoo lanza se puede integrar de manera mas rapida y economica, permitiendole capitalizar la innovacion continua."
