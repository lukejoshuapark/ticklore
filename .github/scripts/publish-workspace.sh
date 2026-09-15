#!/usr/bin/env bash
# Publishes every workspace crate whose current version is not yet on
# crates.io.
#
# Crates are published in dependency order (as computed from `cargo
# metadata`'s resolve graph, restricted to normal - not dev - dependencies
# between workspace members), and after publishing each one, this script
# waits for it to appear on crates.io before publishing anything that
# depends on it, since that dependent's own verification build resolves the
# dependency from the registry rather than the local path.
#
# Writes one "name version" line per crate actually published to
# $PUBLISHED_LOG, for the workflow's tagging step.
set -euo pipefail

: "${USER_AGENT:?USER_AGENT must be set}"
: "${PUBLISHED_LOG:?PUBLISHED_LOG must be set}"
: > "$PUBLISHED_LOG"

metadata=$(cargo metadata --format-version 1)
members=$(jq -c '.workspace_members' <<<"$metadata")

declare -A pkg_name pkg_version
while IFS=$'\t' read -r id name version; do
  pkg_name["$id"]=$name
  pkg_version["$id"]=$version
done < <(jq -r --argjson members "$members" '
  .packages[] | select(.id as $id | $members | index($id)) | [.id, .name, .version] | @tsv
' <<<"$metadata")

# Direct *normal* (not dev) dependency edges between workspace members:
# dependent id -> dependency id. Dev-dependencies are excluded because
# `cargo publish`'s verification build does not need them, so they should
# not force a publish order.
declare -A deps
for id in "${!pkg_name[@]}"; do deps["$id"]=""; done

while IFS=$'\t' read -r id dep; do
  deps["$id"]+="$dep"$'\n'
done < <(jq -r --argjson members "$members" '
  .resolve.nodes[]
  | select(.id as $id | $members | index($id))
  | .id as $id
  | (.deps[] | select(any(.dep_kinds[]?; .kind == null)) | .pkg) as $dep
  | select($members | index($dep))
  | "\($id)\t\($dep)"
' <<<"$metadata")

# Kahn's algorithm: repeatedly emit members whose dependencies have all
# already been emitted.
declare -A emitted
order=()
remaining=("${!pkg_name[@]}")

while [ "${#remaining[@]}" -gt 0 ]; do
  progressed=false
  next_remaining=()

  for id in "${remaining[@]}"; do
    ready=true

    while IFS= read -r dep; do
      [ -z "$dep" ] && continue
      if [ -z "${emitted[$dep]:-}" ]; then
        ready=false
        break
      fi
    done <<<"${deps[$id]}"

    if $ready; then
      order+=("$id")
      emitted["$id"]=1
      progressed=true
    else
      next_remaining+=("$id")
    fi
  done

  if ! $progressed; then
    echo "::error::Circular dependency detected among workspace crates"
    exit 1
  fi

  remaining=("${next_remaining[@]}")
done

crate_status() {
  curl -sS -o /dev/null -w '%{http_code}' \
    --retry 3 --retry-delay 5 \
    -H "User-Agent: $USER_AGENT" \
    "https://crates.io/api/v1/crates/$1/$2"
}

for id in "${order[@]}"; do
  name=${pkg_name[$id]}
  version=${pkg_version[$id]}

  status=$(crate_status "$name" "$version")
  case "$status" in
    200)
      echo "::notice::$name $version is already published; skipping"
      continue
      ;;
    404)
      echo "::notice::Publishing $name $version"
      ;;
    *)
      echo "::error::Unexpected HTTP $status from crates.io for $name $version"
      exit 1
      ;;
  esac

  cargo publish -p "$name"
  echo "$name $version" >> "$PUBLISHED_LOG"

  echo "::notice::Waiting for $name $version to appear on crates.io"
  indexed=false
  for _ in $(seq 1 24); do
    if [ "$(crate_status "$name" "$version")" = "200" ]; then
      indexed=true
      break
    fi
    sleep 5
  done

  if ! $indexed; then
    echo "::error::$name $version did not appear on crates.io after publishing"
    exit 1
  fi
done
