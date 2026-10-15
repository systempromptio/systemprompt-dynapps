# Pipeline / CI Diagnosis

How to diagnose and fix failed CI/CD pipelines. This methodology reads the repo's actual configuration rather than assuming a specific setup. **Provider-agnostic** — works with GitLab CI and GitHub Actions.

## Step 0: Understand the repo context

Before looking at logs, check what files exist in the repo to understand how it's configured. Not all repos have the same files — read what's there and adapt.

| File | What it tells you | How to use it |
|------|-------------------|---------------|
| `.gitlab-ci.yml` | GitLab CI pipeline definition | If missing, check: `glab api projects/:id --jq .ci_config_path` |
| `.github/workflows/*.yml` | GitHub Actions workflow definitions | Each file is a separate workflow |
| `dyn-build.yaml` | Build manifest — what repos get cloned, which branches, which sources | Check when build fails: wrong branch? missing repo? unreachable source? |
| `.pre-commit-config.yaml` | What linting hooks are active — varies by project and language version | Read when pre-commit fails to understand which hook and its config |
| `requirements.txt` | Python dependencies | Check when pip install fails: version conflicts, typos, missing packages |
| `.copier-answers.yml` | Project metadata — Odoo version, repo type, org name (if generated from a template) | Read to auto-detect project context |
| `CODEOWNERS` | Who owns which files/paths | Use when creating MR/PRs to suggest the right reviewer |

## Step 1: Identify the failed job

### GitLab
```bash
glab ci view
glab ci view --branch $(git branch --show-current)
```

### GitHub
```bash
gh run list --branch $(git branch --show-current) --limit 5
gh run view <run-id>
```

Note which job(s) failed and their stage/step. Multiple failures in the same stage often share a root cause.

## Step 2: Read the job logs

### GitLab
```bash
# Get job ID from glab ci view output, then:
glab ci trace <job-id>
```

### GitHub
```bash
# Get run ID from gh run list, then:
gh run view <run-id> --log
# Or for a specific job:
gh run view <run-id> --log --job <job-id>
```

Focus on the last 50-100 lines — that's where the error usually is. Look for error messages, tracebacks, exit codes, and "FAILED" or "ERROR" markers.

## Step 3: Classify and diagnose

Based on the job name, stage, and error content:

### Pre-commit / Linting failures

**Symptoms:** Job name contains "pre-commit", "lint", or "format". Errors mention tools like ruff, prettier, eslint, pylint, etc.

**Diagnosis:** Read `.pre-commit-config.yaml` to see what hooks are configured. Run `pre-commit run --all-files` locally to reproduce.

**Common fixes:**
- Formatting issues -> `pre-commit run --all-files`, commit the auto-fixes
- Import order -> run the specific tool (e.g., `ruff check --fix .`)
- Linting errors -> read the specific error, fix the code

### Test failures

**Symptoms:** Job name contains "test". Errors show Python tracebacks with `FAIL`, `ERROR`, or `AssertionError`.

**Diagnosis:**
- Read the traceback to identify which test failed and why
- Check if it's a test you wrote (likely a real bug) or an existing test (your code might break existing functionality)
- `setUp` / `setUpClass` failures indicate environment issues, not test logic

**Common fixes:**
- `AssertionError` -> code produces wrong result. Fix code or update test
- `AccessError` -> missing security rules (ir.model.access or ir.rule)
- `ValidationError` -> constraint violation. Check required fields, unique constraints
- `IntegrityError` -> database constraint. Check field definitions, foreign keys
- `ImportError` -> missing dependency in `__manifest__.py` or `requirements.txt`

### Build failures

**Symptoms:** Job name contains "build", "docker", or "image". Errors during Docker build, git clone, pip install.

**Diagnosis:**
- If the error is during git clone of a dependency -> check `dyn-build.yaml` for wrong branch, missing repo, or unreachable source
- If pip install -> check `requirements.txt` for version conflicts
- If Docker COPY -> file referenced in Dockerfile doesn't exist in the repo
- Network timeout -> transient, retry

### Install/Neutralize failures

**Symptoms:** Job name contains "install" or "neutralize". Odoo module installation fails.

**Diagnosis:**
- Look for which module failed to install
- `ParseError` in XML -> malformed view or data file
- `KeyError` or `AttributeError` -> model or field reference error

**Common fixes:**
- XML parse error -> check for unclosed tags or mismatched attributes
- Missing dependency -> add to `depends` in `__manifest__.py`
- Field not found -> check spelling, ensure the model it depends on is installed

### Deploy failures

**Symptoms:** Job in a "deploy" stage. Errors related to Docker push, Helm, webhooks.

**Diagnosis:** Usually infrastructure issues, not code. Escalate to the infra team.

## Step 4: Suggest the fix

After diagnosing, provide:
1. **What failed** — the specific job and error
2. **Why it failed** — root cause
3. **How to fix it** — concrete steps
4. **How to verify** — how to confirm locally before pushing

## Retrying pipelines

Only retry for clearly transient errors (network timeouts, runner allocation). For deterministic failures, fix the code first.

### GitLab
```bash
glab ci retry <job-id>
```

### GitHub
```bash
gh run rerun <run-id>
# Or just the failed jobs:
gh run rerun <run-id> --failed
```

## Local verification before pushing

```bash
# Pre-commit hooks
pre-commit run --all-files

# Odoo tests (run the specific failing test)
python odoo-bin -d test_db --test-tags /module_name -i module_name --stop-after-init
```

This saves time — a pipeline/CI run can take 10+ minutes, local checks take seconds.
