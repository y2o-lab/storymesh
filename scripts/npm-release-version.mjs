const releaseVersionPattern =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-(alpha|beta|rc|canary)\.([1-9]\d*))?$/;

export function distTagForVersion(version) {
  const match = releaseVersionPattern.exec(version);
  if (!match) {
    throw new Error(
      `unsupported release version ${version}; use X.Y.Z or X.Y.Z-{alpha|beta|rc|canary}.N (N >= 1)`,
    );
  }
  return match[4] ?? "latest";
}
