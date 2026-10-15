# Reglas de Normalizacion de Datos

> Reglas para normalizar datos de archivos de importacion antes de enviarlos a Odoo.
> Incluye variantes comunes y XML IDs correspondientes.

---

## Paises (res.country)

### Variantes comunes → XML ID

| Variantes | XML ID Odoo | Nombre oficial |
|-----------|-------------|----------------|
| España, Spain, Espana, ES, españa, spain, ESPAÑA, esp | `base.es` | España |
| Francia, France, FR, francia, france | `base.fr` | Francia |
| Alemania, Germany, DE, alemania, deutschland | `base.de` | Alemania |
| Portugal, PT, portugal | `base.pt` | Portugal |
| Italia, Italy, IT, italia | `base.it` | Italia |
| Reino Unido, UK, United Kingdom, GB, Inglaterra, England, gran bretaña | `base.uk` | Reino Unido |
| Estados Unidos, USA, US, United States, EEUU, EE.UU. | `base.us` | Estados Unidos |
| Mexico, México, MX | `base.mx` | México |
| Colombia, CO | `base.co` | Colombia |
| Argentina, AR | `base.ar` | Argentina |
| Chile, CL | `base.cl` | Chile |
| Peru, Perú, PE | `base.pe` | Perú |
| Brasil, Brazil, BR | `base.br` | Brasil |
| Paises Bajos, Netherlands, Holanda, Holland, NL | `base.nl` | Países Bajos |
| Belgica, Belgium, BE, Bélgica | `base.be` | Bélgica |
| Suiza, Switzerland, CH | `base.ch` | Suiza |
| Austria, AT | `base.at` | Austria |
| Canada, Canadá, CA | `base.ca` | Canadá |
| Marruecos, Morocco, MA | `base.ma` | Marruecos |
| Andorra, AD | `base.ad` | Andorra |
| China, CN | `base.cn` | China |
| Japon, Japan, JP, Japón | `base.jp` | Japón |

### Logica de normalizacion
1. Convertir a minusculas y eliminar acentos
2. Buscar en tabla de variantes
3. Si no hay match exacto → usar fuzzy matching (difflib, threshold 0.7)
4. Si no hay match → marcar como warning y pedir decision al usuario

---

## Provincias España (res.country.state)

| Variantes | XML ID Odoo | Nombre |
|-----------|-------------|--------|
| A Coruña, La Coruña, Coruña | `base.state_es_c` | A Coruña |
| Alava, Álava, Araba | `base.state_es_vi` | Álava |
| Albacete | `base.state_es_ab` | Albacete |
| Alicante, Alacant | `base.state_es_a` | Alicante |
| Almeria, Almería | `base.state_es_al` | Almería |
| Asturias, Oviedo | `base.state_es_o` | Asturias |
| Avila, Ávila | `base.state_es_av` | Ávila |
| Badajoz | `base.state_es_ba` | Badajoz |
| Barcelona, BCN | `base.state_es_b` | Barcelona |
| Burgos | `base.state_es_bu` | Burgos |
| Caceres, Cáceres | `base.state_es_cc` | Cáceres |
| Cadiz, Cádiz | `base.state_es_ca` | Cádiz |
| Cantabria, Santander | `base.state_es_s` | Cantabria |
| Castellon, Castellón, Castelló | `base.state_es_cs` | Castellón |
| Ciudad Real | `base.state_es_cr` | Ciudad Real |
| Cordoba, Córdoba | `base.state_es_co` | Córdoba |
| Cuenca | `base.state_es_cu` | Cuenca |
| Girona, Gerona | `base.state_es_gi` | Girona |
| Granada | `base.state_es_gr` | Granada |
| Guadalajara | `base.state_es_gu` | Guadalajara |
| Guipuzcoa, Guipúzcoa, Gipuzkoa | `base.state_es_ss` | Guipúzcoa |
| Huelva | `base.state_es_h` | Huelva |
| Huesca | `base.state_es_hu` | Huesca |
| Illes Balears, Baleares, Mallorca | `base.state_es_pm` | Illes Balears |
| Jaen, Jaén | `base.state_es_j` | Jaén |
| La Rioja, Rioja | `base.state_es_lo` | La Rioja |
| Las Palmas, Gran Canaria | `base.state_es_gc` | Las Palmas |
| Leon, León | `base.state_es_le` | León |
| Lleida, Lerida, Lérida | `base.state_es_l` | Lleida |
| Lugo | `base.state_es_lu` | Lugo |
| Madrid | `base.state_es_m` | Madrid |
| Malaga, Málaga | `base.state_es_ma` | Málaga |
| Murcia | `base.state_es_mu` | Murcia |
| Navarra | `base.state_es_na` | Navarra |
| Ourense, Orense | `base.state_es_or` | Ourense |
| Palencia | `base.state_es_p` | Palencia |
| Pontevedra | `base.state_es_po` | Pontevedra |
| Salamanca | `base.state_es_sa` | Salamanca |
| Santa Cruz de Tenerife, Tenerife | `base.state_es_tf` | Santa Cruz de Tenerife |
| Segovia | `base.state_es_sg` | Segovia |
| Sevilla | `base.state_es_se` | Sevilla |
| Soria | `base.state_es_so` | Soria |
| Tarragona | `base.state_es_t` | Tarragona |
| Teruel | `base.state_es_te` | Teruel |
| Toledo | `base.state_es_to` | Toledo |
| Valencia, València | `base.state_es_v` | Valencia |
| Valladolid | `base.state_es_va` | Valladolid |
| Vizcaya, Bizkaia | `base.state_es_bi` | Vizcaya |
| Zamora | `base.state_es_za` | Zamora |
| Zaragoza | `base.state_es_z` | Zaragoza |
| Ceuta | `base.state_es_ce` | Ceuta |
| Melilla | `base.state_es_ml` | Melilla |

---

## Telefonos

### Reglas de normalizacion
1. Eliminar espacios, guiones, parentesis, puntos
2. Si empieza por `00` → reemplazar por `+`
3. Si no tiene prefijo y el pais del contacto es España → añadir `+34`
4. Formato final recomendado: `+34612345678`

### Ejemplos
| Original | Normalizado |
|----------|-------------|
| 612 345 678 | +34612345678 |
| (612) 345-678 | +34612345678 |
| 0034 612345678 | +34612345678 |
| +34 612 345 678 | +34612345678 |
| 91 234 56 78 | +34912345678 |
| +1 (555) 123-4567 | +15551234567 |

---

## Fechas

### Formato destino Odoo
- `Date`: `YYYY-MM-DD` (ej: `2026-03-14`)
- `Datetime`: `YYYY-MM-DD HH:MM:SS` (ej: `2026-03-14 10:30:00`)

### Conversion desde formatos comunes
| Formato origen | Ejemplo | Conversion |
|---------------|---------|------------|
| DD/MM/YYYY | 14/03/2026 | 2026-03-14 |
| DD-MM-YYYY | 14-03-2026 | 2026-03-14 |
| DD.MM.YYYY | 14.03.2026 | 2026-03-14 |
| MM/DD/YYYY | 03/14/2026 | 2026-03-14 (detectar y advertir) |
| YYYY-MM-DD | 2026-03-14 | 2026-03-14 (ya correcto) |
| D/M/YY | 14/3/26 | 2026-03-14 |

### Ambiguedad MM/DD vs DD/MM
- Si hay fechas con dia > 12 → confirma formato europeo (DD/MM)
- Si todos los valores son ≤12 en ambas posiciones → **preguntar al usuario**
- Regla por defecto para archivos en español: DD/MM/YYYY

---

## Numeros y Monedas

### Reglas
| Formato origen | Normalizado | Notas |
|---------------|-------------|-------|
| 1.234,56 (europeo) | 1234.56 | Separador miles: punto, decimal: coma |
| 1,234.56 (anglosajón) | 1234.56 | Separador miles: coma, decimal: punto |
| 1234,56 | 1234.56 | Sin separador miles, decimal: coma |
| 1 234,56 | 1234.56 | Separador miles: espacio |
| 1.234€ | 1234.0 | Eliminar simbolo moneda |
| $1,234.56 | 1234.56 | Eliminar simbolo moneda |

### Deteccion automatica
- Si hay coma como ultimo separador y 2 digitos despues → decimal europeo
- Si hay punto como ultimo separador y 2 digitos despues → decimal anglosajón
- Si ambiguo → usar la convencion del pais del archivo (España → europeo)

---

## NIF/CIF/VAT

### Formato Odoo para España
- Formato: `ES` + 9 caracteres (letra/numeros)
- Ejemplo: `ESB12345678`, `ES12345678A`

### Normalizacion
| Original | Normalizado |
|----------|-------------|
| B12345678 | ESB12345678 |
| ES-B12345678 | ESB12345678 |
| ES B12345678 | ESB12345678 |
| 12345678A | ES12345678A |
| b12345678 | ESB12345678 |

### Regla
1. Eliminar espacios y guiones
2. Convertir a mayusculas
3. Si no empieza por `ES` y el pais es España → añadir `ES`
4. Validar formato (no calcular digito de control, solo formato basico)

---

## Booleanos

### Valores aceptados → True/False
| Valores True | Valores False |
|-------------|--------------|
| True, true, TRUE | False, false, FALSE |
| 1 | 0 |
| Si, Sí, si, sí, SI | No, no, NO |
| Yes, yes, YES | No, no, NO |
| X, x | *(vacio)* |

---

## Idiomas (res.lang)

### Codigos comunes
| Variantes | Codigo Odoo |
|-----------|-------------|
| Español, Spanish, Castellano, ES | `es_ES` |
| Ingles, English, EN | `en_US` |
| Frances, French, FR | `fr_FR` |
| Aleman, German, DE | `de_DE` |
| Portugues, Portuguese, PT | `pt_PT` |
| Catalan, Català, CA | `ca_ES` |
| Euskera, Basque, EU | `eu_ES` |
| Gallego, Galician, GL | `gl_ES` |
