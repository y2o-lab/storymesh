export const fixtures = { preact: 'preact', solid: 'solid', qwik: 'qwik', analog: 'angular' };

export function selectFixtures(changed) {
  if (changed.some(path => /^(src\/scanner\/|src\/cli\/|src\/domain\/framework|src\/lib\.rs$|Cargo\.(toml|lock)$|src\/spec\/generation|tests\/generation|scripts\/(check-fixture-builds|fixture-plan)|npm\/test\/fixture-plan|mise\.toml$|\.github\/workflows\/ci\.yml$)/.test(path))) {
    return Object.keys(fixtures);
  }
  return Object.keys(fixtures).filter(name => changed.some(path =>
    path.startsWith(`tests/fixtures/${name}/`) && !path.endsWith('.md')));
}
