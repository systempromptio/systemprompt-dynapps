# Ecosistema Indaws

## 1. Indaws Cloud - Infraestructura

### Alianza con Sarenet
- Data centers en España (Madrid, Pais Vasco, Barcelona, Valencia, Andalucia)
- Servidores VPS estables con alta disponibilidad
- Doble anillo de seguridad (dos firewalls)

### Modalidades de Alojamiento

| Plan | Usuarios | vCPU | RAM | SSD | Workers |
|------|----------|------|-----|-----|---------|
| Pequeno | 0-100 | 2 | 7 GB | 60 GB | 4 |
| Mediano | 101-200 | 4 | 12 GB | 100 GB | 8 |
| Grande | 201-300 | 6 | 20 GB | 120 GB | 12 |

*Staging ilimitados en todos los planes. 1 worker reservado para staging.*

### Certificaciones de Seguridad
- Red Nacional de SOC (categoria Oro)
- ISO/IEC 27001
- SI-0082/2020 AENOR
- Climate Neutral Data Centre
- ASOTEM
- Miembro de MANRS

### Stack Tecnologico

**Docker**: Sistema de contenedores para homogeneidad. Entornos de produccion y pruebas totalmente aislados. Imagenes optimizadas con gestion adaptativa de dependencias.

**Traefik**: Proxy inverso nativo para Docker.
- Proteccion de URLs criticas (admin DB)
- Filtrado por IP
- Certificados SSL automaticos (Let's Encrypt: HTTP-01, TLS-ALPN-01, DNS-01)
- Renovacion automatica
- Entornos de pruebas securizados al mismo nivel que produccion

**Let's Encrypt**: Certificados SSL gratuitos. Soporte para HTTP-01, TLS-ALPN-01 y DNS-01 (con infraestructura ACME-DNS propia).

**Tailscale**: Tunel SSH seguro para servidores detras de firewall. Sin necesidad de abrir puertos, sin comprometer la seguridad.

**Cloudflare**:
- Dominios *.indaws.cloud con proteccion por defecto
- WAF (Web Application Firewall) adaptado a Odoo
- Zero Trust para accesos privados
- Almacenamiento R2 (compatible S3) para adjuntos y backups

### Copias de Seguridad
- Sistema Cloudflare R2
- Copia diaria de los ultimos 10 dias (personalizable)
- Solo entorno de produccion
- Sarenet: snapshot completo diario del servidor

### Dominios
- Acceso via *.indaws.cloud
- Configuracion de dominios propios sin limites
- Proteccion WAF de Cloudflare bajo demanda

### Almacenamiento Ampliado
- Cloudflare R2 para proyectos con muchos adjuntos o uso intenso de Documentos
- Sin limite de almacenamiento
- Sin paradas programadas para ampliacion

---

## 2. Indaws Cloud - Plataforma

Orquestador de Odoo propio desarrollado por Indaws (https://www.indaws.cloud).
- Seguridad y flexibilidad en despliegues
- Reduccion de fallos humanos
- Integra Docker, Traefik, Let's Encrypt, Tailscale, Cloudflare

---

## 3. Paquetes Adicionales

### Seguridad Avanzada (650 EUR)
Compatible con: Indaws Cloud, On-premise, Odoo Online*, Odoo.sh*
- Despliegue de WAF en dominios del cliente
- Filtrado de trafico no deseado (reglas adaptadas a Odoo)
- Filtrado por IP para usuarios internos
- Zero Trust opcional
- Hasta 5 reglas WAF personalizadas
- Cloudflare Turnstile para formularios web
- Doble factor de autenticacion de Odoo

---

## 4. Indaws Core - Capa de Valor

### Que Es
Capa de soluciones curadas sobre Odoo estandar. No personalizaciones aleatorias, sino soluciones desarrolladas para resolver **problemas de negocio reales** identificados a lo largo de la experiencia de Indaws.

### Beneficios
- Mantenida y migrada directamente por Indaws
- Estabilidad y garantia superiores a personalizaciones aisladas
- Facilita enormemente las futuras actualizaciones
- Potencia Odoo para admitir crecimiento

### Modulos Core Principales
- `indaws_storage`: Gestion de almacenamiento base
- `indaws_storage_s3`: Conector para Amazon S3 / Cloudflare R2
- `indaws_report_docx`: Motor de informes en formato PDF/Word

### Arquitectura de 3 Capas
```
┌──────────────────────────────┐
│        Indaws Core           │  ← Capa de valor mantenida por Indaws
├──────────────────────────────┤
│      Odoo Enterprise         │  ← Base solida del fabricante
├──────────────────────────────┤
│  Personalizaciones cliente   │  ← Solo si Core/Studio no cubren
└──────────────────────────────┘
```

---

## 5. Metodologia de Migracion (3 Fases)

### Fase 1: Analisis, Optimizacion y Migracion Tecnica (8 semanas)
**Objetivo**: Base de datos migrada, optimizada y lista para pruebas (sin datos reales).

1. **Clonado del Sistema**: Copia segura sin impactar operativa diaria
2. **Auditoria de Personalizaciones**: Para cada una, decision estrategica:
   - a) ¿Es necesaria? → Si obsoleta, eliminar
   - b) ¿Puede sustituirse por Indaws Core? → Reemplazar
   - c) ¿Puede reconstruirse con Studio? → Mas sostenible
   - d) ¿Requiere migracion de codigo? → Migrar modulo
3. **Migracion Tecnica**: Migracion de codigo de modulos a medida e Indaws Core
4. **Despliegue de entorno de pruebas**: Validacion tecnica interna

### Fase 2: Migracion de Datos y Pruebas de Aceptacion (6 semanas)
**Objetivo**: Funcionalidad y datos validados por el cliente.

1. **Scripts de migracion de datos**: Sobre copia actualizada
2. **Pruebas de integridad**: Datos migrados correctamente
3. **UAT (Pruebas de Aceptacion)**: El equipo del cliente valida flujos habituales
4. **Ajustes y correcciones**: Resolver incidencias del UAT

### Fase 3: Puesta en Marcha y Soporte (2 semanas)
**Objetivo**: Nuevo sistema en produccion con transicion fluida.

1. **Planificacion del Go-Live**: Fin de semana para minimizar impacto
2. **Migracion Definitiva**: Ultima migracion de datos 100% actualizada
3. **Puesta en Produccion**: Activacion del nuevo entorno
4. **Soporte Post-Arranque**: Soporte intensivo primeros dias

### Cronograma Total: ~16 semanas (4 meses)

---

## 6. Estrategia de Personalizaciones

### Prioridad (de mas a menos sostenible)
1. **Odoo Studio**: Campos, vistas modificadas. Se migran automaticamente. No requieren scripts. SIEMPRE la primera opcion.
2. **Indaws Core**: Soluciones mantenidas por Indaws. Migacion garantizada. Segunda opcion.
3. **Modulos OCA**: Dependen de terceros. Riesgo de retraso en migracion. Evitar si hay alternativa estandar o Core.
4. **Modulos a Medida**: Coste recurrente de mantenimiento y migracion. Ultima opcion, solo si no hay alternativa.
