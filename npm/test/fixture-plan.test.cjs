const { test } = require('node:test');
const assert = require('node:assert/strict');

test('fixture selection covers shared generator changes and isolates fixture changes', async () => {
  const { fixtures, selectFixtures } = await import('../../scripts/fixture-plan.mjs');
  const all = Object.keys(fixtures);
  for (const path of ['src/scanner/generation.rs', 'src/scanner/mod.rs',
    'src/spec/generation.rs', 'src/cli/run.rs', 'src/domain/framework.rs', 'Cargo.lock', 'tests/generation.rs', 'scripts/check-fixture-builds.mjs',
    'scripts/fixture-plan.mjs', '.github/workflows/ci.yml', 'mise.toml']) {
    assert.deepEqual(selectFixtures([path]), all, path);
  }
  assert.deepEqual(selectFixtures(['tests/fixtures/preact/src/Card.jsx']), ['preact']);
  assert.deepEqual(selectFixtures(['tests/fixtures/qwik/package-lock.json', 'tests/fixtures/analog/.storybook/main.ts']), ['qwik', 'analog']);
  assert.deepEqual(selectFixtures(['README.md', 'tests/fixtures/preact/README.md', 'src/report.rs']), []);
});
