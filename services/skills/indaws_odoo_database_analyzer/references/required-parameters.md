# Required Parameters

## Question 1: Client/Project
| Parameter | Description | Required |
|-----------|-------------|----------|
| `CLIENT_NAME` | Client or project name for the report header | Yes |

## Question 2: Project Paths
| Parameter | Description | Required |
|-----------|-------------|----------|
| `MODULE_PATHS` | Client's custom/legacy module paths (comma-separated) | Yes |
| `INDAWS_CORE_PATHS` | Indaws Core source paths | Only for Refactoring types |

## Question 3: Version Information
| Parameter | Options | Required |
|-----------|---------|----------|
| `SOURCE_VERSION` | v14, v15, v16, v17, v18, v19 | Yes |
| `TARGET_VERSION` | v15, v16, v17, v18, v19 (must be > SOURCE) | Yes |

## Question 4: Project Scope
| Parameter | Options | Default |
|-----------|---------|---------|
| `PROJECT_SIZE` | Small (+10% risk), Medium (+18% risk), Large (+25% risk) | Medium |

## Question 5: Migration Type
| Type | Description | Available Solutions |
|------|-------------|---------------------|
| `1:1` | Direct migration without refactoring | Standard, Custom only |
| `Refactoring` | Full optimization with all options | Standard, Studio, Core, Custom |
| `Refactoring_No_Studio` | Optimization without Studio | Standard, Core, Custom |

## Question 6: Database Migration (Optional)
| Parameter | Description | Default |
|-----------|-------------|---------|
| `INCLUDE_DB_MIGRATION` | Include database upgrade estimation | No |
| `DB_SIZE_GB` | Database size in gigabytes | 5 |

## Question 7: Functional Testing (Optional)
| Parameter | Description | Default |
|-----------|-------------|---------|
| `INCLUDE_FUNCTIONAL_TESTING` | Include functional testing estimation by business consultant | No |

## Question 8: Unit Testing (Optional)
| Parameter | Description | Default |
|-----------|-------------|---------|
| `INCLUDE_UNIT_TESTING` | Include AI-assisted unit test development estimation for Custom migrated modules | No |

## Question 9: Output Configuration
| Parameter | Description | Required |
|-----------|-------------|----------|
| `OUTPUT_PATH` | Full path with filename (.md) | Yes |
| `LANGUAGE` | English, Spanish, Catalan | Auto-detect |
| `DEEP_ANALYSIS` | Enable code quality analysis | Yes (default) |
