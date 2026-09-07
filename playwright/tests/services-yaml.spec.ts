// Descriptor readers must accept both equivalent YAML block-sequence styles.
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test, expect } from '@playwright/test';
import { includeList } from './support/services-yaml';

for (const indent of ['    ', '      ']) {
  test(`descriptor include lists accept ${indent.length}-space items`, () => {
    const dir = mkdtempSync(join(tmpdir(), 'astound-yaml-test-'));
    try {
      const file = join(dir, 'config.yaml');
      writeFileSync(file, [
        'plugin:', '  skills:', '    source: explicit', '    include:',
        `${indent}- first_skill`, `${indent}- second_skill`,
        '  agents:', '    source: explicit', '    include: []', '',
      ].join('\n'));
      expect(includeList(file, 'skills')).toEqual(['first_skill', 'second_skill']);
      expect(includeList(file, 'agents')).toEqual([]);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
}
