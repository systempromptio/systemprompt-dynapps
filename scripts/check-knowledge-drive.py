#!/usr/bin/env python3
"""Validate Drive ingestion with the local schema and isolated integration databases."""
import json
import os
from pathlib import Path
import subprocess
import urllib.parse

root = Path(__file__).resolve().parent.parent
os.chdir(root)
secrets = json.loads((root / '.systemprompt/profiles/local/secrets.json').read_text())
url = secrets['database_url']
env = os.environ.copy()
env['DATABASE_URL'] = url
env['SQLX_OFFLINE'] = 'false'
env['SQLX_OFFLINE_DIR'] = str(root / '.sqlx')
parts = urllib.parse.urlsplit(url)
env['PGDATABASE'] = urllib.parse.unquote(parts.path.lstrip('/'))
env['PGHOST'] = parts.hostname or 'localhost'
env['PGPORT'] = str(parts.port or 5432)
env['PGUSER'] = urllib.parse.unquote(parts.username or '')
env['PGPASSWORD'] = urllib.parse.unquote(parts.password or '')
env['SYSTEMPROMPT_TEST_DATABASE_URL'] = urllib.parse.urlunsplit((parts.scheme, parts.netloc, '/postgres', '', ''))
for name in ['knowledge_bank.sql', 'google_drive.sql']:
    subprocess.run(['psql', '-Xq', '-v', 'ON_ERROR_STOP=1', '-f', str(root / 'extensions/mcp/knowledge-bank/schema' / name)], env=env, check=True)
commands = [
    ['cargo', 'clippy', '-p', 'systemprompt-mcp-knowledge-bank', '-p', 'systemprompt-web-jobs', '--', '-D', 'warnings'],
    ['cargo', 'nextest', 'run', '--manifest-path', 'tests/Cargo.toml', '-p', 'mcp-unit-tests', '-E', 'test(knowledge_drive) | test(knowledge_bank)'],
    ['cargo', 'nextest', 'run', '--manifest-path', 'tests/Cargo.toml', '-p', 'mcp-integration-tests', '-E', 'test(knowledge_drive) | test(knowledge_retrieval) | test(knowledge_files)'],
]
for command in commands:
    print('Running:', ' '.join(command), flush=True)
    subprocess.run(command, env=env, check=True)
