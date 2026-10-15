# OCA Repository Catalog

All repos use `https://github.com/OCA/{repo}.git` as base URL.

## Core Repos (117) — Cloned automatically

### Contabilidad / Finanzas (19)
| Repo | Description |
|------|-------------|
| account-analytic | Analytic accounting |
| account-budgeting | Budget management |
| account-closing | Period closing |
| account-consolidation | Multi-company consolidation |
| account-financial-reporting | Financial reports (balance, P&L) |
| account-financial-tools | Accounting utilities |
| account-fiscal-rule | Fiscal position rules |
| account-invoice-reporting | Invoice reports |
| account-invoicing | Invoice workflows |
| account-payment | Payment management |
| account-reconcile | Bank reconciliation |
| bank-payment | Banking payment orders (SEPA) |
| bank-payment-alternative | Alternative payment methods |
| bank-statement-import | Bank statement import (OFX, CAMT) |
| credit-control | Credit control & dunning |
| currency | Currency tools |
| margin-analysis | Margin analysis |
| mis-builder | Management Information System reports |
| mis-builder-contrib | MIS Builder community templates |

### Ventas (8)
| Repo | Description |
|------|-------------|
| sale-workflow | Sale order workflows |
| sale-reporting | Sale reports |
| sale-financial | Sale financial tools |
| sale-blanket | Blanket orders |
| sale-channel | Sales channels |
| sale-prebook | Sale prebook |
| sale-promotion | Promotions & discounts |
| commission | Sales commissions |

### Compras (2)
| Repo | Description |
|------|-------------|
| purchase-workflow | Purchase workflows |
| purchase-reporting | Purchase reports |

### Stock / Logística (13)
| Repo | Description |
|------|-------------|
| stock-logistics-warehouse | Warehouse management |
| stock-logistics-workflow | Stock workflows |
| stock-logistics-barcode | Barcode scanning |
| stock-logistics-reporting | Stock reports |
| stock-logistics-tracking | Tracking & tracing |
| stock-logistics-transport | Transport management |
| stock-logistics-availability | Stock availability |
| stock-logistics-interfaces | External system interfaces |
| stock-logistics-orderpoint | Reordering rules |
| stock-logistics-putaway | Putaway strategies |
| stock-logistics-release-channel | Release channels |
| stock-logistics-request | Stock requests |
| stock-logistics-reservation | Stock reservation |

### Delivery (1)
| Repo | Description |
|------|-------------|
| delivery-carrier | Delivery carrier integrations |

### WMS (1)
| Repo | Description |
|------|-------------|
| wms | Warehouse Management System |

### Fabricación (2)
| Repo | Description |
|------|-------------|
| manufacture | Manufacturing extensions |
| manufacture-reporting | Manufacturing reports |

### RRHH (4)
| Repo | Description |
|------|-------------|
| hr | Human Resources |
| hr-attendance | Attendance tracking |
| hr-expense | Expense management |
| hr-holidays | Leave management |

### Timesheet (2)
| Repo | Description |
|------|-------------|
| hr-timesheet | Timesheet extensions |
| timesheet | Timesheet tools |

### Proyecto (3)
| Repo | Description |
|------|-------------|
| project | Project management |
| project-agile | Agile/Scrum project |
| project-reporting | Project reports |

### CRM (1)
| Repo | Description |
|------|-------------|
| crm | CRM extensions |

### Contratos / Acuerdos (3)
| Repo | Description |
|------|-------------|
| contract | Recurring contracts |
| agreement | Agreements management |
| program | Loyalty/program management |

### EDI (4)
| Repo | Description |
|------|-------------|
| edi | Electronic Data Interchange |
| edi-ediversa | EDI Ediversa connector |
| edi-framework | EDI framework base |
| edi-voxel | EDI Voxel connector |

### Web / Website (5)
| Repo | Description |
|------|-------------|
| web | Backend web UI extensions |
| web-api | REST/JSON API |
| web-api-contrib | API community contributions |
| website | Website builder extensions |
| website-cms | CMS extensions |

### Server / Backend (6)
| Repo | Description |
|------|-------------|
| server-tools | Server utilities |
| server-ux | UX improvements |
| server-auth | Authentication modules |
| server-backend | Backend connectors |
| server-env | Environment configuration |
| server-brand | Branding/debranding |

### Reporting (3)
| Repo | Description |
|------|-------------|
| reporting-engine | Report engines (xlsx, py3o) |
| report-print-send | Print & send reports |
| webkit-tools | Webkit report tools |

### Mail / Social (2)
| Repo | Description |
|------|-------------|
| mail | Mail extensions |
| social | Social media |

### Partner (1)
| Repo | Description |
|------|-------------|
| partner-contact | Partner/contact extensions |

### RMA / Reparación (3)
| Repo | Description |
|------|-------------|
| rma | Return Merchandise Authorization |
| repair | Repair management |
| tier-validation | Tier validation (approval workflows) |

### DMS / Storage (2)
| Repo | Description |
|------|-------------|
| dms | Document Management System |
| storage | File storage backends |

### Localización España (1)
| Repo | Description |
|------|-------------|
| l10n-spain | Spanish localization (SII, AEAT, banks) |

### Productos (5)
| Repo | Description |
|------|-------------|
| product-attribute | Product attributes |
| product-configurator | Product configurator |
| product-kitting | Product kitting |
| product-pack | Product packs |
| product-variant | Product variants |

### Supply Chain (1)
| Repo | Description |
|------|-------------|
| ddmrp | Demand Driven MRP |

### Varios funcional (10)
| Repo | Description |
|------|-------------|
| brand | Multi-brand management |
| business-requirement | Business requirements |
| calendar | Calendar extensions |
| data-protection | GDPR / data protection |
| donation | Donation management |
| geospatial | GIS / geospatial |
| maintenance | Maintenance management |
| multi-company | Multi-company rules |
| operating-unit | Operating units |
| resource | Resource management |

### Otros técnicos (5)
| Repo | Description |
|------|-------------|
| queue | Job queue (OCA/queue) |
| role-policy | Role-based access policies |
| search-engine | Search engine connectors |
| shoppingfeed | Shopping feed connector |
| survey | Survey extensions |

### Webhook / Themes (2)
| Repo | Description |
|------|-------------|
| webhook | Webhook management |
| website-themes | Website themes |

---

## On-Demand Repos (142) — Available but NOT cloned automatically

Clone individually when needed:
```bash
git clone -b {version} --depth 1 https://github.com/OCA/{repo}.git
```

### Tooling / Infra (32)
`OCB`, `.github`, `maintainer-tools`, `maintainer-quality-tools`, `oca-ci`, `oca-github-bot`, `oca-port`, `oca-weblate-deployment`, `oca-addons-repo-template`, `oca-custom`, `oca-decorators`, `oca.recipe.odoo`, `pylint-odoo`, `odoo-pre-commit-hooks`, `odoo-sphinx-autodoc`, `odoo-module-migrator`, `odoo-sentinel`, `odoorpc`, `odoo-test-helper`, `openupgradelib`, `OpenUpgrade`, `contribute-md-template`, `repo-maintainer`, `repo-maintainer-conf`, `mirrors-flake8`, `py3o.template`, `docs`, `dotnet`, `odoo-community.org`, `connector-magento-php-extension`, `module-composition-analysis`, `version-control-platform`

### Localizaciones — excepto España (49)
`l10n-argentina`, `l10n-austria`, `l10n-belarus`, `l10n-belgium`, `l10n-brazil`, `l10n-bulgaria`, `l10n-cambodia`, `l10n-canada`, `l10n-chile`, `l10n-china`, `l10n-colombia`, `l10n-costa-rica`, `l10n-croatia`, `l10n-ecuador`, `l10n-estonia`, `l10n-ethiopia`, `l10n-finland`, `l10n-france`, `l10n-germany`, `l10n-greece`, `l10n-india`, `l10n-indonesia`, `l10n-iran`, `l10n-ireland`, `l10n-italy`, `l10n-japan`, `l10n-luxemburg`, `l10n-macedonia`, `l10n-mexico`, `l10n-morocco`, `l10n-netherlands`, `l10n-norway`, `l10n-paraguay`, `l10n-peru`, `l10n-poland`, `l10n-portugal`, `l10n-romania`, `l10n-russia`, `l10n-slovenia`, `l10n-switzerland`, `l10n-taiwan`, `l10n-thailand`, `l10n-turkey`, `l10n-ukraine`, `l10n-united-kingdom`, `l10n-uruguay`, `l10n-usa`, `l10n-venezuela`, `l10n-vietnam`

### Verticales (15)
`vertical-abbey`, `vertical-agriculture`, `vertical-association`, `vertical-community`, `vertical-construction`, `vertical-cooperative-supermarket`, `vertical-edition`, `vertical-education`, `vertical-hotel`, `vertical-isp`, `vertical-medical`, `vertical-ngo`, `vertical-realestate`, `vertical-rental`, `vertical-travel`

### Conectores (18)
`connector`, `connector-accountedge`, `connector-cmis`, `connector-ecommerce`, `connector-infor`, `connector-interfaces`, `connector-jira`, `connector-lengow`, `connector-lims`, `connector-magento`, `connector-odoo2odoo`, `connector-prestashop`, `connector-redmine`, `connector-sage`, `connector-salesforce`, `connector-spscommerce`, `connector-telephony`, `connector-woocommerce`

### Cubiertos por Enterprise (21)
`helpdesk`, `field-service`, `sign`, `spreadsheet`, `payroll`, `fleet`, `e-learning`, `event`, `mass-mailing`, `e-commerce`, `shopfloor-app`, `stock-logistics-shopfloor`, `pos`, `knowledge`, `automation`, `shift-planning`, `crowdfunding`, `cooperative`, `cim`, `infrastructure`, `wallet`

### Varios excluidos (7)
`apps-store`, `ansible-odoo`, `odoo-pim`, `pwa-builder`, `rest-api`, `rest-framework`, `iot`
