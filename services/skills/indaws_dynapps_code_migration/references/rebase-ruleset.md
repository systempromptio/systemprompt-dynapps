# Rebase Conflict Ruleset

Rules for resolving conflicts during `git rebase <oldver>/develop` (Step 1).

## Manifest conflicts: `xx_base/__manifest__.py`

- **`depends`**: keep ALL entries from the old version + add any new ones from the new template.
- **`version`**: take the NEW version string (e.g. `"18.0.1.0.0"`).
- All other keys: take the new version.

## Manifest conflicts: `xx_all/__manifest__.py`

- **`depends`**: keep ALL entries from the old version but **comment every module out**:
  ```python
  "depends": [
      "xx_base",
      # "dynapps_invoicing",
      # "dynapps_jira_connector",
      # ...
  ],
  ```
- **`version`**: take the NEW version string.
- All other keys: take the new version.

## Files: always take NEW version (no manual merge needed)

| File/pattern |
|---|
| `.copier-answers.yml` |
| `.coveragerc` |
| `.gitignore` |
| `.pre-commit-config.yaml` |
| `.pylintrc` |
| `.pylintrc-mandatory` |
| `requirements.*.txt` |
| `config/server_environment_files/**` |

## Files/directories: DELETE after rebase

Remove these from the working tree (they belong to the old version scaffold):

```
_tools/
_deploy_*.sh
*.cfg              (odoo config files at root level)
VERSION.txt
upgrade.py
requirements-buildout.txt
requirements-odoo.txt
.gitlab-ci.yml
.eslintrc.yml
.flake8
.isort.cfg
.prettierrc.yml
```

```bash
rm -rf _tools/ _deploy_*.sh *.cfg VERSION.txt upgrade.py \
       requirements-buildout.txt requirements-odoo.txt \
       .gitlab-ci.yml .eslintrc.yml .flake8 .isort.cfg .prettierrc.yml
```

## ir.config_parameter entries

Any production `ir.config_parameter` key found in the old repo →
add to `default/ir_config_parameter.conf` with `value = False`:

```ini
[your.param.key]
value = False
```

## Pre-commit exclude block (Step 3)

After rebase, when marking modules non-installable, add them to `.pre-commit-config.yaml`:

```yaml
exclude: |
  (?x)
  # Custom folders to ignore during upgrade
  parts/custom/dynapps_invoicing|
  parts/custom/dynapps_jira_connector
```

One module per line, `|`-separated, last line has no trailing `|`.

## Conflict resolution escalation

If a conflict does not match any rule above:
1. Show the full diff hunk to the user.
2. AskUserQuestion: "This conflict has no automatic ruleset. Which version should I take?"
3. Never guess on business logic conflicts.
