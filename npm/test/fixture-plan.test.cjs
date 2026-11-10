const { test } = require('node:test');
const assert = require('node:assert/strict');

test('fixture selection covers shared generator changes and isolates fixture changes', async () => {
  const { fixtures, selectFixtures } = await import('../../scripts/fixture-plan.mjs');
  const all = Object.keys(fixtures);
  for (const path of ['src/scanner/generation.rs', 'src/scanner/mod.rs',
    'src/spec/generation.rs', 'src/cli/run.rs', 'src/domain/framework.rs', 'src/domain/coverage.rs', 'Cargo.lock', 'tests/generation.rs', 'scripts/check-fixture-builds.mjs',
    'scripts/fixture-plan.mjs', '.github/workflows/ci.yml', 'mise.toml']) {
    assert.deepEqual(selectFixtures([path]), all, path);
  }
  assert.deepEqual(selectFixtures(['tests/fixtures/preact/src/Card.jsx']), ['preact']);
  assert.deepEqual(selectFixtures(['tests/fixtures/qwik/package-lock.json', 'tests/fixtures/analog/.storybook/main.ts']), ['qwik', 'analog']);
  assert.deepEqual(selectFixtures(['README.md', 'tests/fixtures/preact/README.md', 'src/report.rs']), []);
});

test('fixture verification requires Card even when Button was generated', async () => {
  const { verifyExpectedStories } = await import('../../scripts/fixture-plan.mjs');
  for (const [name, extension] of [['preact', 'jsx'], ['solid', 'tsx'], ['qwik', 'tsx']]) {
    const button = `src/Button.stories.${extension}`;
    const card = `src/Card.stories.${extension}`;
    assert.throws(() => verifyExpectedStories(name, [button]), /Card.*was not generated/);
    assert.throws(() => verifyExpectedStories(name, [button, `other/Card.stories.${extension}`]), /Card.*was not generated/);
    assert.doesNotThrow(() => verifyExpectedStories(name, [button, card]));
    assert.throws(() => verifyExpectedStories(name, []), /no generated stories/);
  }
  assert.doesNotThrow(() => verifyExpectedStories('analog', ['src/app/pages/index.page.stories.ts']));
  assert.throws(() => verifyExpectedStories('analog', []), /no generated stories/);
});
