#!/usr/bin/env bash
# Release preflight: every file that carries the release version must agree
# (and match the tag) before anything is built, hosted or published.
#
# Usage:
#   bash .github/scripts/check-release-versions.sh v0.7.0   # before pushing a release tag
#   bash .github/scripts/check-release-versions.sh          # consistency only (PRs)
#
# release.yml runs this in its `plan` job, so a mismatch fails the release in
# about a minute instead of after the GitHub Release and some registries have
# already gone out.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

TAG="${1:-}"
errors=0

err() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then echo "::error::$1"; else echo "error: $1" >&2; fi
  errors=$((errors + 1))
}

# Print the `version = "..."` value of a TOML section, e.g. [workspace.package].
toml_version() { # <file> <section>
  awk -v section="[$2]" '
    { sub(/\r$/, "") }
    $0 == section { in_section = 1; next }
    /^\[/ { in_section = 0 }
    in_section && /^version *=/ { gsub(/^version *= *"|".*$/, ""); print; exit }
  ' "$1"
}

# Print the version of a workspace package as recorded in Cargo.lock.
lock_version() { # <package>
  awk -v name="name = \"$1\"" '
    { sub(/\r$/, "") }
    $0 == name { getline; sub(/\r$/, ""); gsub(/^version = "|"$/, ""); print; exit }
  ' Cargo.lock
}

expected="$(toml_version Cargo.toml workspace.package)"
if [ -z "$expected" ]; then
  err "could not read [workspace.package] version from Cargo.toml"
  exit 1
fi
echo "Release version (Cargo.toml [workspace.package]): $expected"

check() { # <label> <actual>
  if [ "$2" = "$expected" ]; then
    printf '  ok        %-50s %s\n' "$1" "$2"
  else
    printf '  MISMATCH  %-50s %s\n' "$1" "${2:-<missing>}"
    err "$1 is '${2:-<missing>}', expected '$expected'"
  fi
}

if [ -n "$TAG" ]; then
  check "git tag $TAG" "${TAG#v}"
fi
check "Cargo.toml [workspace.dependencies] slimg-core" \
  "$(sed -nE 's/^slimg-core *= *\{.*version *= *"([^"]+)".*/\1/p' Cargo.toml | tr -d '\r')"
for pkg in slimg slimg-core slimg-ffi; do
  check "Cargo.lock $pkg" "$(lock_version "$pkg")"
done
check "bindings/python/pyproject.toml [project]" \
  "$(toml_version bindings/python/pyproject.toml project)"
check "bindings/kotlin/gradle.properties" \
  "$(sed -n 's/^version=//p' bindings/kotlin/gradle.properties | tr -d '\r')"

# Re-pushing a tag whose GitHub Release already exists would rebuild
# everything for ~40 minutes and then fail at `gh release create`.
if [ -n "$TAG" ] && [ -n "${GITHUB_ACTIONS:-}" ]; then
  if gh release view "$TAG" --repo "$GITHUB_REPOSITORY" > /dev/null 2>&1; then
    err "GitHub Release $TAG already exists. If only a publish step failed, run the 'Publish' workflow manually with tag=$TAG instead of re-tagging (see docs/releasing.md)."
  fi
fi

if [ "$errors" -gt 0 ]; then
  echo
  echo "Preflight failed with $errors problem(s)."
  if [ -n "$TAG" ]; then
    echo "Nothing has been built or published yet: fix the files, commit, then move the tag:"
    echo "  git tag -f $TAG && git push -f origin $TAG"
  fi
  exit 1
fi
echo "Preflight passed."
