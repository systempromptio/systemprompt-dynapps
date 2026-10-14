# Conocimiento de Sectores para Demos Realistas

Este documento contiene informacion detallada de sectores industriales para generar datos de demo realistas en Odoo. Cada sector incluye nomenclatura tipica, rangos de precios, categorias y tipos de cliente.

---

## Sector: Fabricacion de Muebles

### Caracteristicas del Sector
- Productos con variantes (modelos, colores, materiales, dimensiones)
- Nomenclatura descriptiva (modelo + caracteristicas fisicas)
- Mezcla B2B (hoteles, oficinas, contract) y B2C
- Apps habituales: Ventas, Inventario, Fabricacion, Compras, Contabilidad, CRM

### Nomenclatura Tipica
**Formato**: [Tipo] [Modelo] [Caracteristicas] [Material/Acabado]

**Ejemplos**:
- Sofa Modelo Valencia 3 plazas tela gris antracita
- Mesa comedor roble macizo 180x90 cm extensible
- Silla oficina ergonomica con reposabrazos negra
- Armario puertas correderas 200x180 blanco mate
- Estanteria modular 5 baldas melamina roble

### Categorias
- Sofas y sillones
- Mesas (comedor, centro, auxiliares)
- Sillas y taburetes
- Armarios y almacenaje
- Camas y cabeceros
- Estanterias y librerias

### Rangos de Precio
- Sillas: 50 - 500 EUR
- Mesas: 150 - 2.000 EUR
- Sofas: 500 - 3.000 EUR
- Armarios: 300 - 2.500 EUR
- Camas: 200 - 1.500 EUR

### Clientes Tipicos
**B2B**: Hotel Plaza Mayor, Oficinas Modernas S.L., Restaurante El Rincon, Decoraciones Martinez
**B2C**: Sr./Sra. [Apellido]

### Codigo Producto: [TIPO][MODELO]-[VARIANTE] (ej: SOF-VAL-3P-GRS)

---

## Sector: Distribucion Alimentaria

### Caracteristicas del Sector
- Alto volumen de referencias (miles de productos)
- Productos perecederos y no perecederos, lotes y caducidad
- Cadena de frio para algunos productos
- Apps habituales: Ventas, Compras, Inventario, Contabilidad, Punto de Venta

### Nomenclatura Tipica
**Formato**: [Marca/Generico] [Descripcion] [Formato] [Cantidad]

**Ejemplos**:
- Aceite de oliva virgen extra 1L botella cristal
- Conserva de tomate natural 500g lata
- Leche entera UHT 1L brick (pack 6 uds)
- Jamon iberico loncheado 100g envasado al vacio
- Harina de trigo integral 1kg bolsa papel

### Categorias
- Aceites y conservas
- Lacteos
- Carnes y embutidos
- Frutas y verduras
- Panaderia y pasteleria
- Bebidas
- Congelados

### Rangos de Precio
- Conservas: 1 - 15 EUR
- Aceites: 5 - 40 EUR
- Lacteos: 0.80 - 5 EUR
- Embutidos: 3 - 50 EUR
- Bebidas: 0.50 - 20 EUR

### Clientes Tipicos
Supermercado El Sol, Tienda de alimentacion La Huerta, Restaurante La Buena Mesa, Hotel Costa Azul, Catering Eventos del Sur, Residencia Tercera Edad Los Olivos

### Codigo Producto: [CAT][SUBCATEGORIA]-[NUM] (ej: ACE-OLV-001)

---

## Sector: Servicios IT / Consultoria

### Caracteristicas del Sector
- Productos = Servicios (horas, licencias, proyectos)
- Precios por hora o proyecto cerrado
- Clientes principalmente B2B
- Apps habituales: CRM, Ventas, Proyecto, Contabilidad, Helpdesk, RRHH

### Nomenclatura Tipica
**Formato**: [Tipo servicio] [Especialidad] [Nivel]

**Ejemplos**:
- Consultoria ERP Odoo - Jefe de Proyecto (hora)
- Desarrollo backend Python - Senior (hora)
- Analisis funcional - Consultor (jornada)
- Soporte tecnico nivel 2 - Mensual
- Licencia Odoo Enterprise (usuario/mes)

### Categorias
- Consultoria
- Desarrollo
- Soporte
- Formacion
- Licencias

### Rangos de Precio
- Consultor junior: 40 - 60 EUR/hora
- Consultor senior: 60 - 90 EUR/hora
- Desarrollador: 50 - 80 EUR/hora
- Jefe de proyecto: 80 - 120 EUR/hora
- Proyectos cerrados: 5.000 - 100.000+ EUR

### Clientes Tipicos
Construcciones Garcia S.L., Clinica Dental Dr. Martinez, Grupo Empresarial del Sur, Ayuntamiento de Alcala

### Codigo Servicio: [AREA]-[TIPO]-[NIVEL] (ej: CONS-ERP-SR)

---

## Sector: Hotel / Hosteleria

### Caracteristicas del Sector
- Habitaciones como producto principal + servicios adicionales
- Tarifas dinamicas segun temporada/ocupacion
- Apps habituales: Ventas, Punto de Venta, Contabilidad, RRHH, Website, Suscripciones

### Nomenclatura Tipica
**Habitaciones**: Habitacion Individual estandar, Habitacion Doble vista mar, Suite Junior 40m2 con terraza
**Servicios**: Desayuno buffet, Media pension, Parking cubierto (dia), Spa circuito termal

### Categorias
- Alojamiento
- Restauracion
- Servicios adicionales
- Eventos y reuniones

### Rangos de Precio
- Hab. individual: 60 - 150 EUR/noche
- Hab. doble: 80 - 250 EUR/noche
- Suite: 150 - 500 EUR/noche
- Desayuno: 8 - 20 EUR
- Evento salon: 500 - 3.000 EUR

### Clientes Tipicos
**B2C**: Sr./Sra. [Apellido], Familia [Apellido]
**B2B**: Agencia Viajes Sol, Empresa Tours del Mediterraneo, Corporacion Eventos Globales

### Codigo: [TIPO]-[CARACTERISTICA] (ej: HAB-IND-STD, SRV-DESAYUNO)

---

## Sector: Talleres / Automocion

### Caracteristicas del Sector
- Mezcla productos (recambios) y servicios (mano de obra)
- Ordenes de reparacion, gestion de flotas B2B
- Apps habituales: Ventas, Compras, Inventario, Contabilidad, CRM, Helpdesk

### Nomenclatura Tipica
**Recambios**: Bosch 0242236562 Bujia, Mann HU7008Z Filtro aceite, Castrol Edge 5W-30 4L
**Servicios**: Cambio de aceite y filtros, ITV pre-inspeccion, Diagnosis electronica

### Categorias
**Productos**: Filtros, Aceites, Neumaticos, Frenos, Baterias, Iluminacion
**Servicios**: Mantenimiento, Reparacion mecanica, Reparacion electrica, Chapa y pintura

### Rangos de Precio
- Filtros: 5 - 30 EUR
- Aceites: 20 - 80 EUR
- Mano de obra: 40 - 70 EUR/hora
- Reparaciones: 100 - 2.000+ EUR

### Clientes Tipicos
**B2C**: Sr./Sra. [Apellido]
**B2B**: Transportes del Norte S.L., Rent a Car Express, Flota Municipal

### Codigo: referencia fabricante para recambios, [TIPO]-[AREA]-[NUM] para servicios

---

## Sector: Construccion / Materiales

### Caracteristicas del Sector
- Productos a granel y unidades, gestion de obras como proyectos
- Ventas por pales, sacos, m3
- Apps habituales: Ventas, Compras, Inventario, Contabilidad, Proyecto

### Nomenclatura Tipica
**Formato**: [Material] [Especificacion] [Formato/Unidad]

**Ejemplos**:
- Cemento Portland CEM II 42,5 saco 25kg
- Ladrillo ceramico perforado 25x12x7 cm
- Arena de rio lavada m3 a granel
- Panel aislante poliestireno XPS 50mm (m2)

### Rangos de Precio
- Cemento: 5 - 10 EUR/saco
- Ladrillos: 0.20 - 1 EUR/ud
- Arena/grava: 15 - 30 EUR/m3
- Aislamiento: 5 - 20 EUR/m2

### Clientes Tipicos
Constructora Levante S.L., Reformas Martinez, Arquitecto Fernandez Asociados

### Codigo: [CATEGORIA]-[TIPO]-[ESPECIF] (ej: CEM-PORT-425)

---

## Sector: Oil & Gas / Energia

### Caracteristicas del Sector
- Productos energeticos (combustibles, lubricantes) + servicios tecnicos
- Alto control regulatorio, certificaciones
- Apps habituales: Ventas, Compras, Inventario, Contabilidad, Proyecto, Fabricacion

### Nomenclatura Tipica
- Gasoleo A (litro), Gas natural GLP (kg), Aceite hidraulico ISO 46 (bidón 20L)
- Servicio mantenimiento planta solar, Auditoria energetica certificada

### Rangos de Precio
- Combustibles: 0.80 - 1.80 EUR/litro
- Lubricantes industriales: 50 - 200 EUR/bidon
- Servicios tecnicos: 2.000 - 50.000 EUR/proyecto

### Clientes Tipicos
Planta Solar del Sur S.A., Petrolinea Distribuciones, Central Termica Norte

---

## Sector: Farmacia / Salud

### Caracteristicas del Sector
- Productos regulados (lotes, trazabilidad, caducidad)
- Mezcla productos y servicios sanitarios
- Apps habituales: Ventas, Compras, Inventario, Contabilidad, Website, Suscripciones

### Nomenclatura Tipica
- Ibuprofeno 600mg comprimidos caja 40 uds
- Guantes latex talla M caja 100 uds
- Servicio analisis clinico completo
- Silla de ruedas plegable aluminio

### Rangos de Precio
- Medicamentos OTC: 3 - 30 EUR
- Material sanitario: 5 - 500 EUR
- Equipamiento medico: 100 - 10.000 EUR

### Clientes Tipicos
Farmacia Central, Clinica Salud Total, Residencia Los Pinos, Hospital Comarcal

---

## Sector: Retail / Moda

### Caracteristicas del Sector
- Variantes por talla, color, temporada
- Gestion de temporadas y colecciones
- Apps habituales: Ventas, Punto de Venta, Inventario, Website/eCommerce, Compras

### Nomenclatura Tipica
- Camiseta basica algodon M/L/XL blanca
- Pantalon chino slim fit T42 azul marino
- Zapatillas deportivas running T39 gris
- Bolso bandolera cuero marron

### Rangos de Precio
- Camisetas: 15 - 60 EUR
- Pantalones: 30 - 120 EUR
- Calzado: 40 - 200 EUR
- Complementos: 15 - 150 EUR

### Clientes Tipicos
**B2C**: consumidores finales
**B2B**: Boutique Elegance, Grandes Almacenes del Sur, Tienda Online ModaYa

---

## Sector: Logistica / Transporte

### Caracteristicas del Sector
- Servicios de transporte (nacional/internacional), almacenaje
- Gestion de flotas, rutas, entregas
- Apps habituales: Ventas, Inventario, Contabilidad, Proyecto, RRHH

### Nomenclatura Tipica
- Transporte nacional carga completa (Madrid-Barcelona)
- Servicio grupaje paqueteria (<30kg)
- Almacenaje palet/mes zona frio
- Servicio aduanero importacion

### Rangos de Precio
- Envio nacional palet: 80 - 300 EUR
- Envio internacional: 200 - 2.000 EUR
- Almacenaje: 8 - 25 EUR/palet/mes
- Grupaje: 5 - 50 EUR/bulto

### Clientes Tipicos
Distribuciones Rapidas S.L., eCommerce Express, Industrias del Norte S.A.

---

## Sector: Agricultura / Agroalimentaria

### Caracteristicas del Sector
- Productos agricolas (semillas, fertilizantes, maquinaria)
- Estacionalidad marcada, dependencia climatica
- Apps habituales: Ventas, Compras, Inventario, Contabilidad, Fabricacion

### Nomenclatura Tipica
- Semilla trigo duro certificada saco 40kg
- Fertilizante NPK 15-15-15 saco 25kg
- Aceite de oliva virgen extra produccion propia 5L
- Servicio recoleccion mecanizada (hectarea)

### Rangos de Precio
- Semillas: 30 - 150 EUR/saco
- Fertilizantes: 20 - 60 EUR/saco
- Aceite produccion: 15 - 40 EUR/5L
- Servicios mecanizados: 100 - 500 EUR/ha

### Clientes Tipicos
Cooperativa Agricola del Valle, Finca Los Olivos, Bodega Tierra Alta

---

## Sector: Educacion / Formacion

### Caracteristicas del Sector
- Productos = cursos, matriculas, materiales didacticos
- Suscripciones (mensualidades), eventos (jornadas)
- Apps habituales: Ventas, CRM, Website, Contabilidad, Suscripciones, Proyecto

### Nomenclatura Tipica
- Curso presencial Marketing Digital 40h
- Matricula Master MBA Online 2025-2026
- Material didactico modulo Contabilidad
- Pack 10 sesiones coaching ejecutivo

### Rangos de Precio
- Cursos cortos: 100 - 500 EUR
- Formacion bonificada: 200 - 2.000 EUR
- Masters: 3.000 - 15.000 EUR
- Coaching: 80 - 200 EUR/sesion

### Clientes Tipicos
**B2C**: particulares (alumnos)
**B2B**: Empresa Formacion Plus S.L., Departamento RRHH Corporacion XYZ, Fundacion Educativa del Sur

---

## Convenciones Generales

### Nombres de Empresas (B2B)
- [Actividad] [Apellido] (ej: Construcciones Garcia)
- [Actividad] [Nombre] S.L. (ej: Distribuciones Mediterraneo S.L.)
- Grupo [Nombre] (ej: Grupo Empresarial del Sur)
- [Sector] [Caracteristica] (ej: Hotel Vista Mar)

### Direcciones Tipicas (Espana)
- Calle Mayor 23, 28013 Madrid
- Avda. del Puerto 45, 46021 Valencia
- Poligono Industrial Las Americas, Nave 12, 41006 Sevilla
- Paseo de la Castellana 89, 28046 Madrid

### Emails: info@[empresa].com, comercial@[empresa].es
### Telefonos: Madrid 91X, Barcelona 93X, Valencia 96X, Sevilla 95X

---

## Como Usar Este Documento

1. **Identifica el sector** mas aproximado al cliente
2. **Adapta la nomenclatura**: usa los formatos como guia, no literal
3. **Respeta los rangos**: precios y cantidades realistas
4. **Genera variedad**: no copies los ejemplos, genera variaciones coherentes
5. **Coherencia**: productos, clientes y transacciones deben tener sentido juntos

**Ejemplo de coherencia**:
- CORRECTO: Un "Hotel Plaza Mayor" compra 20 "Sofas Modelo Valencia" → tiene sentido
- INCORRECTO: Un "Sr. Garcia" (particular) compra 500 "Sacos de cemento 25kg" → poco realista para B2C
