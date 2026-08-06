#!/usr/bin/env bash
# Fix up the release PR after an egui update.
#
# An egui update is a breaking change for every crate, but the update commit
# doesn't touch the individual crates, so release-plz proposes patch bumps for
# them. This script runs on the release PR branch: every published crate whose
# pending bump is only patch-level gets the next minor version instead, and
# "Update egui to x.y" is added to its changelog section.
#
# Idempotent: crates whose pending version already increases the minor (or
# more) are left alone, and nothing happens when the workspace egui version
# matches the version the crates were last published against (checked via the
# crates.io index).
#
# Used by .github/workflows/release-plz.yml on every push to main; can also be
# run locally on a checked-out release PR branch.
# Requires: jq, curl, release-plz (https://release-plz.dev).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

if ! git diff --quiet; then
    echo "error: working tree has uncommitted changes" >&2
    exit 1
fi

# Sparse index URL for a crate (name must be at least 4 characters long).
index_url() {
    local name=$1
    echo "https://index.crates.io/${name:0:2}/${name:2:2}/${name}"
}

# Latest published version of a crate; empty if it was never published.
published_version() {
    curl -fsSL "$(index_url "$1")" 2>/dev/null |
        jq -rs '[.[] | select(.yanked | not)] | last | .vers // empty' || true
}

# "major.minor" of a version or version requirement like "^0.36.0".
major_minor() {
    sed -E 's/^[^0-9]*([0-9]+\.[0-9]+).*/\1/' <<<"$1"
}

metadata=$(cargo metadata --no-deps --format-version 1)

# egui_dnd's egui requirement stands in for the whole workspace.
local_egui=$(major_minor "$(jq -r '.packages[]
    | select(.name == "egui_dnd")
    | [.dependencies[] | select(.name == "egui")][0].req' <<<"$metadata")")
published_egui=$(major_minor "$(curl -fsSL "$(index_url egui_dnd)" |
    jq -rs '[.[] | select(.yanked | not)] | last
        | [.deps[] | select(.name == "egui")][0].req')")

if [[ "$local_egui" == "$published_egui" ]]; then
    echo "egui $local_egui matches the published crates, nothing to do"
    exit 0
fi
echo "egui was updated: $published_egui -> $local_egui"

bumps=()    # crate@version arguments for release-plz set-version
changelogs=()   # "path<TAB>version" of the changelog sections to edit

while IFS=$'\t' read -r name version manifest; do
    published=$(published_version "$name")
    if [[ -z "$published" ]]; then
        echo "$name: not published yet, skipping"
    elif [[ "$version" == "$published" ]]; then
        echo "$name: no pending release, skipping"
    elif [[ "$(major_minor "$version")" != "$(major_minor "$published")" ]]; then
        echo "$name: already bumped to $version (published: $published)"
    else
        next="$(cut -d. -f1 <<<"$published").$(($(cut -d. -f2 <<<"$published") + 1)).0"
        echo "$name: $version -> $next"
        bumps+=("$name@$next")
        changelogs+=("$(dirname "$manifest")/CHANGELOG.md	$next")
    fi
done < <(jq -r '.packages[] | select(.publish == null)
    | [.name, .version, .manifest_path] | @tsv' <<<"$metadata")

if [[ ${#bumps[@]} -eq 0 ]]; then
    echo "all pending releases already bump the minor version, nothing to do"
    exit 0
fi

# Updates the versions, the version requirements of dependent crates, and
# renames the pending changelog sections.
release-plz set-version "${bumps[@]}"

# Name the egui update in the changelog sections: without a commit that touches
# the crate, release-plz only writes a generic "update Cargo.toml dependencies"
# entry for it. Sections that already mention the egui update (from a real
# commit that touched the crate) are left alone.
for entry in "${changelogs[@]}"; do
    path=${entry%	*}
    version=${entry#*	}
    if awk -v header="## $version" '
        $0 == header { insection = 1; next }
        insection && /^## / { exit }
        insection && /^- Update egui/ { found = 1; exit }
        END { exit !found }
    ' "$path"; then
        continue
    fi
    awk -v header="## $version" -v entry="- Update egui to $local_egui" '
        $0 == header { print; print ""; print entry; insection = 1; skipblank = 1; next }
        insection && /^## / { insection = 0 }
        insection && $0 == "- update Cargo.toml dependencies" { next }
        skipblank { skipblank = 0; if ($0 == "") next }
        { print }
    ' "$path" > "$path.tmp"
    mv "$path.tmp" "$path"
done

cargo update --workspace --quiet
