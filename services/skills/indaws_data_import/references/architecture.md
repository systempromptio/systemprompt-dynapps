<!-- verified: v18.0 @ 2026-06 -->
# Architecture Overview

## Models Created by indaws_data_import

```
indaws.import.template          # Reusable import configuration
    ├── line_ids                # Field mappings (One2many → indaws.import.template.line)
    │     ├── match=True        # Mark line as match field (Boolean)
    │     └── code              # Python code for field transformation
    ├── match_line_ids          # Match criteria (One2many → indaws.import.template.match.line)
    ├── match_field_ids         # Computed Many2many from line_ids.filtered(match)
    ├── variable_ids            # Python variables
    ├── parser_id               # File parser
    ├── chunksize               # Records per chunk (default 500)
    ├── commit_every            # Commit every N records (default 0 = end of chunk)
    ├── defer_compute           # Defer field recomputation (Boolean)
    ├── continue_error          # Continue on error (Boolean)
    ├── search_in_archived      # Search matches in archived records (Boolean)
    ├── context_code            # Python code to set import context (returns dict)
    ├── before_import_code      # Python code run once before import starts
    ├── before_create_update_code  # Python code run before each create/update
    ├── finish_import_code      # Python code run once after import ends
    └── group_ids               # Security groups for import execution

indaws.import                   # Import execution
    ├── template_id             # References template
    ├── file                    # Attached file
    ├── job_ids                 # Queue jobs (async)
    ├── error_ids               # Import errors
    └── state                   # draft/enqueued/progress/done

indaws.import.scheduler         # Scheduled imports
    ├── template_id             # Template to execute
    ├── cron_id                 # Odoo cron
    ├── data_collector_id       # Data source (Many2one)
    ├── data_collector_ids      # Data sources (deprecated, use data_collector_id)
    ├── connections_ids         # Connections (Many2many)
    ├── connection_id           # SFTP/API connection
    ├── chunk_size              # Override template chunksize
    ├── last_execution_date     # Last successful execution datetime
    ├── interval_type           # Cron interval type (minutes/hours/days/weeks/months)
    └── interval_number         # Cron interval number

indaws.import.data.collector    # External data sources
    ├── python_code             # Fetch data logic
    ├── param_ids               # Connection parameters (deprecated, use connection)
    └── variable_ids            # Many2many → indaws.import.variable (injected into eval context)

indaws.import.connection        # External connections
    ├── variable_id             # Many2one → indaws.import.variable (auth logic)
    └── param_ids               # Credentials/URL (injected into variable eval context)

indaws.import.variable          # Reusable variables
    ├── name                    # Variable name (injected into eval context under this key)
    ├── python_code             # Code to evaluate (must set 'value')
    └── description             # Documentation
```

## Data Flow: Connection -> Variable -> Collector

```
Connection (e.g., "Provet Cloud OAuth")
    ├── param_ids: client_id, client_secret, base_url, token_url
    └── variable_id → Variable "provet_oauth_token"
            │  python_code: fetches OAuth token, sets value = {base_url, headers}
            │
            ▼
Collector "Provet Cloud - Species"
    ├── connection_id → provides connection variables (provet_oauth_token)
    ├── variable_ids → provides collector-specific variables (e.g., helper functions)
    │
    │  In get_data(): (connection.variables | self.variable_ids).get_variables(eval_ctx)
    │  → All variables evaluated and injected as flat names into collector's python_code context
    │
    └── python_code: uses provet_oauth_token dict + helper functions directly by name
```

**CRITICAL:** Connection params are injected into the variable's eval context (not the collector's).
The variable processes them and its `value` output is injected into the collector context under `variable.name`.

## Streaming with yield_data()

For large datasets (>5K records), collectors should use `yield_data()` instead of
accumulating all data in memory:

```
Collector python_code:
    buffer = []
    for record in api_paginate():
        buffer.append(record)
        if len(buffer) >= YIELD_SIZE:
            yield_data(buffer)  # Flush to scheduler → creates import + processes
            buffer = []
    if buffer:
        yield_data(buffer)  # Flush remaining
```

Each `yield_data()` call triggers the scheduler to create a new `indaws.import` record
and process it immediately, keeping memory usage constant regardless of dataset size.