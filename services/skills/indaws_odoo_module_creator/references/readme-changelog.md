# README.rst & CHANGELOG.md — Templates and Rules

## README.rst

Every module MUST include a `README.rst` following the Dynapps standard format.
The README describes the module to users who haven't read the code: what it does, what it doesn't, and how to configure it.

> **Indaws Core modules** (`indaws_*`, legacy brand profile — see `dynapps-guidelines.md`
> "Brand profile") keep the **Indaws** maintainer block instead: logo/target
> `https://www.indaws.es` and maintainer "Indaws Business Solutions S.L." Standard
> Dynapps modules use the block shown below.

### Template

> **CRITICAL — title rule (RST):** the `=` overline and underline must each contain **exactly
> the same number of characters as the title text** — count them, never off by one. A longer
> rule renders but is wrong for our modules. The title is the module display name **without**
> the `Dynapps:` / `inDAWS:` prefix. Example — `Per-User IoT Printer` is 20 chars, so each
> rule is exactly 20 `=`:
>
> ```
> ====================
> Per-User IoT Printer
> ====================
> ```

```rst
{'=' * len(module_display_name)}   # ← emit len(title) '=' chars, exactly (not one more/less)
{module_display_name}
{'=' * len(module_display_name)}

.. image:: https://img.shields.io/static/v1.svg?label=license&message=LGPL%20v3&color=blue
   :target: https://www.gnu.org/licenses/lgpl-3.0
   :alt: License: LGPL v3

.. image:: https://img.shields.io/static/v1.svg?label=maturity&message=Beta&color=yellow
   :target: https://www.dynapps.es/
   :alt: Maturity: Beta

{One paragraph describing what the module does and why it exists.
Be specific: mention the models it affects, the workflow it changes, and who benefits.}

What this module does NOT do
============================

{Clarify scope boundaries so users don't expect features that aren't included.
Use a bulleted list. Example:}

* Does not handle [related but out-of-scope feature]
* Does not replace [standard module X], it extends it

Configuration
=============

To configure this module, you need to:

#. {Step-by-step configuration instructions}
#. {Each step as a numbered item}
#. {Include menu paths: Settings > Technical > ...}

Usage
=====

To use this module, you need to:

#. {Step-by-step usage instructions}
#. {Describe the main workflow}
#. {Mention any new menu entries, buttons, or views}

Known Issues / Roadmap
======================

* {Any known limitations}
* {Planned improvements}

Credits
=======

Contributors
------------

* {Author Name} <{email}>

Maintainer
----------

.. image:: https://portal.dynapps.es/logo.png
   :alt: dynapps
   :target: https://www.dynapps.es/

This repository is maintained by Dynapps.

To contribute to this repository, please visit https://www.dynapps.es/contact
```

### Section rules

| Section | Required | When to include |
|---------|----------|-----------------|
| Title + badges | Always | Every module |
| Description paragraph | Always | Every module |
| What this module does NOT do | Recommended | When scope could be ambiguous |
| Configuration | If applicable | When the module needs setup beyond install |
| Usage | If applicable | When the workflow isn't obvious from the menu |
| Known Issues / Roadmap | Optional | When there are known limitations |
| Credits | Always | Every module |

### Writing guidelines

- **Description**: Write for a functional consultant, not a developer. Avoid mentioning field names or model names unless necessary.
- **Configuration**: Use menu paths (`Settings > Technical > ...`), not XML IDs.
- **Usage**: Describe the workflow from the user's perspective, not the code's perspective.
- **Maturity badge**: Use `Beta` for new modules. Change to `Production/Stable` only after production validation.
- **License badge**: Match the `license` field in `__manifest__.py`. Default is LGPL-3.

---

## CHANGELOG.md

Every module MUST include a `CHANGELOG.md` in the module root. This file tracks all changes over time: creation, feature additions, bug fixes, and upgrades.

### Template (new module)

```markdown
# Changelog

All notable changes to this module will be documented in this file.

## [{odoo_version}.1.0.0] - {YYYY-MM-DD}

### Added
- Initial release
- {Feature 1: brief description}
- {Feature 2: brief description}
- {Feature N: brief description}

**Author:** {name} <{email}>
```

### Entry format (subsequent changes)

When modifying an existing module, prepend a new entry at the top (below the header):

```markdown
## [{new_version}] - {YYYY-MM-DD}

### {Change type}
- {Brief description of what changed and why}

**Author:** {name} <{email}>
```

### Change types

Use these section headers (based on [Keep a Changelog](https://keepachangelog.com)):

| Type | When to use |
|------|-------------|
| `Added` | New feature or functionality |
| `Changed` | Modification to existing functionality |
| `Fixed` | Bug fix |
| `Removed` | Removed feature or deprecated code |
| `Migrated` | Version upgrade (e.g., 17.0 → 18.0) |
| `Security` | Security-related change |

### Rules

1. **Date format**: Always `YYYY-MM-DD`
2. **Version**: Must match `__manifest__.py` version
3. **Author**: Full name + email of the person who made the change
4. **One entry per logical change**: Don't batch unrelated changes in the same entry
5. **Newest first**: Most recent changes at the top
6. **Be specific**: "Added partner validation on sale confirmation" not "Updated sale module"
7. **On module creation**: Always create CHANGELOG.md with the initial `Added` entry listing all features
8. **On module modification**: Always add a new entry before making code changes

### Example (module with history)

```markdown
# Changelog

All notable changes to this module will be documented in this file.

## [18.0.1.2.0] - 2026-03-28

### Fixed
- Fixed tax computation on multi-currency invoices when partner has fiscal position

**Author:** Toni Hermosilla <toni@dynapps.es>

## [18.0.1.1.0] - 2026-03-15

### Added
- Added automatic partner validation before sale order confirmation
- Added warning message when partner has incomplete fiscal data

**Author:** Brenda Velásquez <brenda@dynapps.es>

## [18.0.1.0.0] - 2026-03-01

### Added
- Initial release
- Sale order extension with custom fiscal validation
- Partner fiscal data completeness check
- Automated email notification on validation failure

**Author:** Toni Hermosilla <toni@dynapps.es>
```
