export const fixtures = { preact: 'preact', solid: 'solid', qwik: 'qwik', analog: 'angular' };

export function selectFixtures(changed) {
  if (changed.some(path => /^(src\/scanner\/|src\/cli\/|src\/domain\/(framework|coverage)|src\/lib\.rs$|Cargo\.(toml|lock)$|src\/spec\/generation|tests\/generation|scripts\/(check-fixture-builds|fixture-plan)|npm\/test\/fixture-plan|mise\.toml$|\.github\/workflows\/ci\.yml$)/.test(path))) {
    return Object.keys(fixtures);
  }
  return Object.keys(fixtures).filter(name => changed.some(path =>
    path.startsWith(`tests/fixtures/${name}/`) && !path.endsWith('.md')));
}

export function verifyExpectedStories(name, storyPaths) {
  if (storyPaths.length === 0) throw new Error(`${name}: no generated stories were checked`);
  const cardStories = {
    preact: 'src/Card.stories.jsx',
    solid: 'src/Card.stories.tsx',
    qwik: 'src/Card.stories.tsx',
  };
  const card = cardStories[name];
  if (card && !storyPaths.includes(card)) throw new Error(`${name}: expected story ${card} was not generated`);
}
