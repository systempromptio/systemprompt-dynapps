# API de Enriquecimiento - Registros Mercantiles Espanoles

## Proveedores soportados

### 1. eInforma (por defecto)

**Web**: https://www.einforma.com
**API Base**: https://api.einforma.com/v1
**Autenticacion**: Bearer token (API key)
**Variable de entorno**: `EINFORMA_API_KEY`

**Endpoint principal**:
```
GET /companies/{cif}
Authorization: Bearer {api_key}
Accept: application/json
```

**Campos devueltos**:
- `name`: Razon social
- `employees`: Numero de empleados
- `turnover`: Facturacion anual (EUR)
- `cnae_code`: Codigo CNAE principal
- `cnae_description`: Descripcion de la actividad
- `province`: Provincia del domicilio social
- `city`: Municipio
- `founded_year`: Ano de constitucion
- `legal_form`: Forma juridica (SA, SL, etc.)
- `status`: Estado (active, inactive, dissolved)

### 2. AXESOR

**Web**: https://www.axesor.es
**API Base**: https://api.axesor.es/v1
**Autenticacion**: X-Api-Key header
**Variable de entorno**: `AXESOR_API_KEY`

**Endpoint principal**:
```
GET /company/{cif}
X-Api-Key: {api_key}
Accept: application/json
```

**Campos devueltos** (dentro de `company`):
- `name`: Razon social
- `employees_number`: Numero de empleados
- `sales_volume`: Volumen de ventas (EUR)
- `cnae`: Codigo CNAE
- `activity_description`: Descripcion actividad
- `province`: Provincia
- `city`: Ciudad
- `constitution_year`: Ano de constitucion
- `legal_form`: Forma juridica
- `status`: Estado

## Formato de CIF/NIF

El script `enrich_leads.py` normaliza automaticamente:
- Prefijo "ES" se elimina (ES + B12345678 -> B12345678)
- Espacios y guiones se eliminan
- Se valida formato CIF (letra + 8 digitos) o NIF (8 digitos + letra)

## Consideraciones

- **Rate limiting**: Delay de 0.5s entre llamadas (configurable con --delay)
- **Coste**: Ambos servicios son de pago. Consultar tarifas vigentes.
- **Fallback**: Si no hay CIF/NIF valido, el lead se marca como `no_valid_vat` y va a revision manual.
- **Modo dry-run**: `--dry-run` simula el enriquecimiento sin llamadas API (para testing).
- **Datos no encontrados**: Empresas nuevas, autonomos o extranjeras pueden no estar en el registro.
