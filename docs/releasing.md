# Releasing

Pushing a `vX.Y.Z` tag runs `.github/workflows/release.yml`:

1. **plan**: preflight version check, then `dist plan`
2. **build**: CLI archives for 5 targets + Homebrew formula (cargo-dist)
3. **host**: creates the GitHub Release with the artifacts
4. **publish** (`publish.yml`): crates.io, Homebrew tap, Maven Central, PyPI

GUI releases (`gui-vX.Y.Z`) are separate (`release-gui.yml`).

## Cutting a release

1. Bump the version everywhere it lives:
   - `Cargo.toml`: `[workspace.package] version` and `[workspace.dependencies] slimg-core`
   - `bindings/python/pyproject.toml`: `[project] version`
   - `bindings/kotlin/gradle.properties`: `version`
   - run `cargo update --workspace` so `Cargo.lock` picks up the new version
2. Check, commit, push:
   ```sh
   bash .github/scripts/check-release-versions.sh vX.Y.Z
   git commit -am "chore: release vX.Y.Z" && git push
   ```
3. Tag and push the tag:
   ```sh
   git tag vX.Y.Z && git push origin vX.Y.Z
   ```

`slimg-libjxl-sys` has its own version. Bump it only when `crates/libjxl-sys`
changes; otherwise the publish job simply skips it.

## When something fails

**Preflight failed (plan job).** Nothing was built or published. Fix the
files, commit, and move the tag:

```sh
git tag -f vX.Y.Z && git push -f origin vX.Y.Z
```

**A build job failed (before host).** No GitHub Release exists yet. If the
failure was flaky, use "Re-run failed jobs". If it needs a code or workflow fix,
fix it on `main` and move the tag as above.

**A publish job failed (after host).** The GitHub Release already exists, so
do **not** re-tag. Every publish step is idempotent (already-published
versions are skipped), so:

- flaky failure (network, registry hiccup): "Re-run failed jobs" on the tag run
- the workflow itself needs a fix: fix `publish.yml` on `main`, then run
  **Actions > Publish > Run workflow** with `tag = vX.Y.Z`. It checks out the
  tag, so it publishes exactly what was tagged with the fixed workflow.

Note that re-running jobs of a tag run always uses the workflow files as they
were at the tagged commit; fixes on `main` only apply through the manual
Publish run.
