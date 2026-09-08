"""Add the requirements runtime to an existing generated Fly profile Dockerfile."""
import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STAGE = '''FROM node:22-bookworm-slim AS requirements-evaluator
WORKDIR /evaluator
COPY services/requirements-workflow/upstream/.project/eval/package*.json ./
RUN npm ci --ignore-scripts --omit=dev --no-audit --no-fund
WORKDIR /projection
COPY services/requirements-workflow/package*.json ./
RUN npm ci --ignore-scripts --omit=dev --no-audit --no-fund

'''
RUNTIME = '''COPY target/release/systemprompt-mcp-requirements-workflow /app/bin/
COPY --from=requirements-evaluator /usr/local/bin/node /usr/local/bin/node
COPY --from=requirements-evaluator /evaluator/node_modules /app/services/requirements-workflow/upstream/.project/eval/node_modules
COPY --from=requirements-evaluator /projection/node_modules /app/services/requirements-workflow/node_modules
COPY scripts/requirements-workflow /app/scripts/requirements-workflow
COPY extensions/mcp/requirements-workflow/manifest.yaml /app/extensions/mcp/requirements-workflow/manifest.yaml
'''


def prepare(profile):
    directory = ROOT / '.systemprompt/profiles' / profile / 'docker'
    dockerfile = directory / 'Dockerfile'
    text = dockerfile.read_text()
    if 'AS requirements-evaluator' not in text:
        if 'USER app' not in text:
            raise RuntimeError('Expected generated runtime USER app boundary')
        text = STAGE + text.replace('USER app', RUNTIME + '\nUSER app', 1)
        dockerfile.write_text(text)
    ignore = directory / 'Dockerfile.dockerignore'
    patterns = ignore.read_text().splitlines()
    for pattern in ['!services/**', '**/node_modules/', 'storage/requirements-workflow/', 'scripts/requirements-workflow/__pycache__/']:
        if pattern not in patterns:
            patterns.append(pattern)
    ignore.write_text('\n'.join(patterns) + '\n')
    print('Prepared requirements runtime for profile', profile, '(no deployment performed)')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('profile', choices=['production'])
    prepare(parser.parse_args().profile)
