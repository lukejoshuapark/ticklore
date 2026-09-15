#!/usr/bin/env bash
# Checks every crate in the workspace against crates.io and reports whether
# any of them have a version that is not published yet.
#
# Writes `publish=true`/`publish=false` to $GITHUB_OUTPUT. Exits non-zero if
# crates.io returns anything other than 200 (published) or 404 (unpublished)
# for any crate, since an ambiguous answer must not silently skip a release
# or trigger an unintended one.
set -euo pipefail

: "${USER_AGENT:?USER_AGENT must be set}"
: "${GITHUB_OUTPUT:?GITHUB_OUTPUT must be set}"

metadata=$(cargo metadata --no-deps --format-version 1)
publish=false

while IFS=$'\t' read -r name version; do
  status=$(curl -sS -o /dev/null -w '%{http_code}' \
    --retry 3 --retry-delay 5 \
    -H "User-Agent: $USER_AGENT" \
    "https://crates.io/api/v1/crates/$name/$version")

  case "$status" in
    404)
      echo "::notice::$name $version is not on crates.io yet"
      publish=true
      ;;
    200)
      echo "::notice::$name $version is already published"
      ;;
    *)
      echo "::error::Unexpected HTTP $status from crates.io for $name $version"
      exit 1
      ;;
  esac
done < <(jq -r '.packages[] | [.name, .version] | @tsv' <<<"$metadata")

echo "publish=$publish" >> "$GITHUB_OUTPUT"
