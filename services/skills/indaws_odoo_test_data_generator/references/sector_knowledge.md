# Conocimiento de Sectores para Generación de Datos Realistas

Este documento contiene información sobre diferentes sectores industriales para ayudar a generar nomenclatura y datos de prueba realistas en Odoo.

---

## Sector: Fabricación de Muebles

### Características del Sector
- Productos con variantes (modelos, colores, materiales, dimensiones)
- Nomenclatura descriptiva (modelo + características físicas)
- Rangos de precio según categoría (sofás 500-3000€, sillas 50-500€)
- Clientes B2B (hoteles, oficinas, tiendas) y B2C

### Nomenclatura Típica de Productos
**Formato**: [Tipo] [Modelo] [Características] [Material/Acabado]

**Ejemplos**:
- Sofá Modelo Valencia 3 plazas tela gris antracita
- Mesa comedor roble macizo 180x90 cm extensible
- Silla oficina ergonómica con reposabrazos negra
- Armario puertas correderas 200x180 blanco mate
- Estantería modular 5 baldas melamina roble

### Categorías Comunes
- Sofás y sillones
- Mesas (comedor, centro, auxiliares)
- Sillas y taburetes
- Armarios y almacenaje
- Camas y cabeceros
- Estanterías y librerías

### Rangos de Precio
- Sillas: 50€ - 500€
- Mesas: 150€ - 2.000€
- Sofás: 500€ - 3.000€
- Armarios: 300€ - 2.500€
- Camas: 200€ - 1.500€

### Tipos de Cliente Típicos
**B2B**:
- Hotel [Nombre] (ej: Hotel Plaza Mayor, Hotel Vista Mar)
- Oficinas [Nombre] S.L. (ej: Oficinas Modernas S.L., Bufete Jurídico García y Asociados)
- Restaurante [Nombre]
- Tienda [Nombre] (ej: Decoraciones Martínez, Muebles El Roble)

**B2C**:
- Nombres de personas (Sr./Sra. [Apellido])

### Códigos de Producto
**Formato**: [TIPO][MODELO]-[VARIANTE]
- Ejemplos: SOF-VAL-3P-GRS, MES-COM-ROB180, SIL-ERG-NEG

---

## Sector: Distribución Alimentaria

### Características del Sector
- Alto volumen de referencias (miles de productos)
- Productos perecederos y no perecederos
- Nomenclatura estándar (marca + formato + contenido)
- Lotes y fechas de caducidad
- Cadena de frío para algunos productos

### Nomenclatura Típica de Productos
**Formato**: [Marca/Genérico] [Descripción] [Formato] [Cantidad]

**Ejemplos**:
- Aceite de oliva virgen extra 1L botella cristal
- Conserva de tomate natural 500g lata
- Leche entera UHT 1L brick (pack 6 uds)
- Jamón ibérico loncheado 100g envasado al vacío
- Harina de trigo integral 1kg bolsa papel

### Categorías Comunes
- Aceites y conservas
- Lácteos
- Carnes y embutidos
- Frutas y verduras
- Panadería y pastelería
- Bebidas
- Congelados

### Rangos de Precio (precio venta)
- Conservas: 1€ - 15€
- Aceites: 5€ - 40€
- Lácteos: 0.80€ - 5€
- Embutidos: 3€ - 50€
- Bebidas: 0.50€ - 20€

### Tipos de Cliente Típicos
- Supermercado [Nombre]
- Tienda de alimentación [Nombre]
- Restaurante [Nombre]
- Hotel [Nombre]
- Catering [Nombre]
- Colectividades (colegios, hospitales, residencias)

### Códigos de Producto
**Formato**: [CAT][SUBCATEGORIA]-[NUM]
- Ejemplos: ACE-OLV-001, CONS-TOM-045, LAC-LEC-ENT-001

---

## Sector: Servicios IT / Consultoría

### Características del Sector
- Productos = Servicios (horas de consultoría, licencias, proyectos)
- Nomenclatura descriptiva orientada a tarea
- Precios por hora o proyecto cerrado
- Clientes principalmente B2B

### Nomenclatura Típica de Servicios
**Formato**: [Tipo servicio] [Especialidad] [Nivel]

**Ejemplos**:
- Consultoría ERP Odoo - Jefe de Proyecto (hora)
- Desarrollo backend Python - Senior (hora)
- Análisis funcional - Consultor (jornada)
- Soporte técnico nivel 2 - Mensual
- Licencia Odoo Enterprise (usuario/mes)

### Categorías Comunes
- Consultoría
- Desarrollo
- Soporte
- Formación
- Licencias

### Rangos de Precio
- Consultor junior: 40€ - 60€/hora
- Consultor senior: 60€ - 90€/hora
- Desarrollador: 50€ - 80€/hora
- Jefe de proyecto: 80€ - 120€/hora
- Proyectos cerrados: 5.000€ - 100.000€+

### Tipos de Cliente Típicos
- [Nombre] S.L. / S.A.
- Ayuntamiento de [Ciudad]
- [Sector] [Nombre] (ej: Clínica Dental Dr. Martínez)
- Grupo [Nombre]

### Códigos de Servicio
**Formato**: [AREA]-[TIPO]-[NIVEL]
- Ejemplos: CONS-ERP-SR, DEV-PY-SR, SOP-NIV2

---

## Sector: Hotel / Hostelería

### Características del Sector
- Habitaciones como producto principal
- Servicios adicionales (restaurante, spa, parking)
- Tarifas dinámicas según temporada/ocupación
- Nomenclatura orientada a tipo de habitación

### Nomenclatura Típica de Productos
**Habitaciones** - Formato: [Tipo] [Características]
- Habitación Individual estándar
- Habitación Doble vista mar
- Suite Junior 40m² con terraza
- Habitación Triple familiar

**Servicios**:
- Desayuno buffet
- Media pensión
- Pensión completa
- Parking cubierto (día)
- Spa circuito termal (sesión)

### Categorías Comunes
- Alojamiento
- Restauración
- Servicios adicionales
- Eventos y reuniones

### Rangos de Precio
- Habitación individual: 60€ - 150€/noche
- Habitación doble: 80€ - 250€/noche
- Suite: 150€ - 500€/noche
- Desayuno: 8€ - 20€
- Comida/cena: 15€ - 40€

### Tipos de Cliente Típicos
**B2C**:
- Sr./Sra. [Apellido]
- Familia [Apellido]

**B2B**:
- Agencia de viajes [Nombre]
- Empresa [Nombre] S.L. (viajes de negocio)
- Touroperador [Nombre]

### Códigos de Producto
**Formato**: [TIPO]-[CARACTERISTICA]-[NUM]
- Ejemplos: HAB-IND-STD, HAB-DOB-VMAR, SRV-DESAYUNO

---

## Sector: Talleres / Automoción

### Características del Sector
- Mezcla de productos (recambios) y servicios (mano de obra)
- Nomenclatura técnica (referencia fabricante)
- Trabajos por OR (orden de reparación)
- Gestión de flotas para clientes B2B

### Nomenclatura Típica
**Recambios** - Formato: [Marca] [Referencia] [Descripción]
- Bosch 0242236562 Bujía
- Mann HU7008Z Filtro aceite
- Castrol Edge 5W-30 Aceite motor sintético 4L
- Brembo P50084 Pastillas freno delanteras

**Servicios**:
- Cambio de aceite y filtros
- ITV pre-inspección
- Cambio de neumáticos + equilibrado
- Reparación sistema de frenos
- Diagnosis electrónica

### Categorías Comunes
**Productos**:
- Filtros
- Aceites y lubricantes
- Neumáticos
- Frenos
- Baterías
- Iluminación

**Servicios**:
- Mantenimiento
- Reparación mecánica
- Reparación eléctrica
- Diagnosis
- Chapa y pintura

### Rangos de Precio
**Productos**:
- Filtros: 5€ - 30€
- Aceites: 20€ - 80€
- Neumáticos: 50€ - 300€ (unidad)
- Baterías: 60€ - 200€

**Servicios**:
- Mano de obra: 40€ - 70€/hora
- Cambio aceite: 30€ - 60€
- ITV: 30€ - 50€
- Reparaciones: 100€ - 2.000€+

### Tipos de Cliente Típicos
**B2C**:
- Sr./Sra. [Apellido]
- [Marca vehículo] [Modelo] [Matrícula]

**B2B**:
- Empresa de transportes [Nombre]
- Flota [Nombre] S.L.
- Rent a Car [Nombre]

### Códigos de Producto/Servicio
- Recambios: usar referencia fabricante
- Servicios: [TIPO]-[AREA]-[NUM]

---

## Sector: Construcción / Materiales

### Características del Sector
- Productos a granel (arena, cemento) y unidades (ladrillos, tejas)
- Gestión de obras como proyectos
- Ventas por palés, sacos, m3
- Transporte incluido o no

### Nomenclatura Típica de Productos
**Formato**: [Material] [Especificación] [Formato/Unidad]

**Ejemplos**:
- Cemento Portland CEM II 42,5 saco 25kg
- Ladrillo cerámico perforado 25x12x7 cm
- Arena de río lavada m3 a granel
- Teja cerámica árabe roja (ud)
- Yeso fino para enlucido saco 25kg

### Categorías Comunes
- Cementos y morteros
- Ladrillos y bloques
- Áridos (arena, grava)
- Tejas y coberturas
- Yesos y escayolas
- Aislamientos
- Herramientas

### Rangos de Precio
- Cemento: 5€ - 10€/saco
- Ladrillos: 0.20€ - 1€/ud
- Arena/grava: 15€ - 30€/m3
- Tejas: 0.50€ - 3€/ud
- Yeso: 4€ - 8€/saco

### Tipos de Cliente Típicos
**B2B**:
- Constructora [Nombre] S.L.
- Reformas [Nombre]
- [Nombre] Construcciones
- Arquitecto [Nombre]

**B2C**:
- Sr./Sra. [Apellido] (obra particular)

### Códigos de Producto
**Formato**: [CATEGORIA]-[TIPO]-[ESPECIF]
- Ejemplos: CEM-PORT-425, LAD-PERF-25, ARN-RIO-LAV

---

## Convenciones Generales para Todos los Sectores

### Nombres de Empresas Cliente (B2B)
**Fórmulas comunes**:
- [Actividad] [Apellido] (ej: Construcciones García)
- [Actividad] [Nombre] S.L. (ej: Distribuciones Mediterráneo S.L.)
- Grupo [Nombre] (ej: Grupo Empresarial del Sur)
- [Nombre] & [Nombre] (ej: Martínez & Asociados)
- [Sector] [Característica] (ej: Hotel Vista Mar, Oficinas Modernas)

### Direcciones Típicas (España)
- Calle Mayor 23, 28013 Madrid
- Avda. del Puerto 45, 46021 Valencia
- Polígono Industrial Las Américas, Nave 12, 41006 Sevilla
- Paseo de la Castellana 89, 28046 Madrid
- Calle Real 156, 15001 A Coruña

### Emails Corporativos
- info@[empresa].com
- comercial@[empresa].com
- ventas@[empresa].es
- pedidos@[empresa].es

### Teléfonos
- Madrid: 91 XXX XX XX
- Barcelona: 93 XXX XX XX
- Valencia: 96 XXX XX XX
- Sevilla: 95 XXX XX XX
- Genérico: 9XX XXX XXX

---

## Cómo Usar Este Documento

1. **Identifica el sector**: Busca el sector que más se aproxime al caso de uso
2. **Adapta la nomenclatura**: Usa los formatos y ejemplos como guía, no como plantilla exacta
3. **Respeta los rangos**: Los precios y cantidades deben estar dentro de rangos realistas
4. **Genera variedad**: No copies los ejemplos literalmente, genera variaciones coherentes
5. **Coherencia**: Asegura que productos, clientes y transacciones tengan sentido juntos

**Ejemplo de coherencia**:
- ✅ CORRECTO: Un "Hotel Plaza Mayor" compra 20 "Sofás Modelo Valencia" → tiene sentido
- ❌ INCORRECTO: Un "Sr. García" (particular) compra 500 "Sacos de cemento 25kg" → poco realista para B2C

---

## Añadir Nuevos Sectores

Si necesitas añadir un nuevo sector a este documento:

1. Investiga la nomenclatura típica del sector (webs de empresas reales, catálogos)
2. Identifica las categorías de productos/servicios más comunes
3. Establece rangos de precio realistas
4. Define formatos de código de producto apropiados
5. Lista tipos de cliente típicos del sector
6. Añade ejemplos concretos y variados

**Mantén la estructura** de las secciones existentes para facilitar la consulta.
