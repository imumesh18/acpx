{
  pkgs,
  ...
}:
let
  cargoAllFeaturesLocked = "--all-features --locked";
  cargoAllTargetsAllFeatures = "--all-targets --all-features";
  oxfmtArgs = ''"." "!CHANGELOG.md"'';
in
{
  languages = {
    rust.enable = true;
    javascript = {
      enable = true;
      npm.enable = true;
    };
  };
  packages = with pkgs; [
    cargo-deny
    cargo-nextest
    git
    git-cliff
    gh
    jq
    just
    oxfmt
    perl
    rsync
    typos
  ];

  scripts = {
    "ref-clone" = {
      description = "Clone or refresh a reference checkout under .ref/.";
      exec = ''
        if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
          echo "Usage: ref-clone <git-url> [name]" >&2
          exit 64
        fi

        url="$1"
        name="''${2:-}"
        if [ -z "$name" ]; then
          name="''${url##*/}"
        fi
        name="''${name%.git}"
        target=".ref/''${name}"

        mkdir -p .ref

        if [ -d "''${target}/.git" ]; then
          git -C "$target" fetch --all --tags --prune
          current_branch="$(git -C "$target" branch --show-current)"
          if [ -n "$current_branch" ]; then
            git -C "$target" pull --ff-only
          fi
        else
          if [ -e "$target" ]; then
            echo "Target '$target' exists and is not a git checkout." >&2
            exit 1
          fi
          git clone "$url" "$target"
        fi

        echo "Reference checkout is available at $target"
      '';
    };

    "ref-copy" = {
      description = "Copy a reference tree or file under .ref/.";
      exec = ''
        if [ "$#" -ne 2 ]; then
          echo "Usage: ref-copy <source> <name>" >&2
          exit 64
        fi

        source_path="$1"
        name="$2"
        target=".ref/''${name}"

        if [ ! -e "$source_path" ]; then
          echo "Source '$source_path' does not exist." >&2
          exit 1
        fi

        mkdir -p .ref "$target"

        if [ -d "$source_path" ]; then
          rsync -a --delete "''${source_path%/}/" "$target/"
        else
          mkdir -p "$target"
          cp "$source_path" "$target/"
        fi

        echo "Reference copy is available at $target"
      '';
    };

    release = {
      description = "Prepare a release commit and annotated tag.";
      exec = ''
        if [ "$#" -gt 1 ]; then
          echo "Usage: release [version]" >&2
          exit 64
        fi

        if ! command -v git-cliff >/dev/null 2>&1; then
          echo "git-cliff is required to prepare a release." >&2
          exit 1
        fi

        if ! command -v jq >/dev/null 2>&1; then
          echo "jq is required to prepare a release." >&2
          exit 1
        fi

        if [ -n "$(git status --porcelain)" ]; then
          echo "Release preparation requires a clean worktree." >&2
          exit 1
        fi

        version="''${1:-}"
        if [ -z "$version" ]; then
          version="$(git cliff --config cliff.toml --bumped-version 2>/dev/null | sed 's/^v//')"
        fi

        if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
          echo "Version '$version' is not a supported SemVer release string." >&2
          exit 1
        fi

        tag="v''${version}"

        if git rev-parse --verify --quiet "$tag" >/dev/null; then
          echo "Tag '$tag' already exists." >&2
          exit 1
        fi

        current_version="$(
          cargo metadata --no-deps --format-version 1 |
            jq -r '.packages[] | select(.name == "acpx") | .version'
        )"

        if [ "$current_version" != "$version" ]; then
          VERSION="$version" perl -0pi -e 's/(\[package\][^\[]*?\nversion = ")[^"]+(")/$1$ENV{VERSION}$2/s' Cargo.toml
          cargo check --all-features >/dev/null
        fi

        git cliff --config cliff.toml --tag "$tag" --output CHANGELOG.md
        devenv --no-tui tasks run quality:check

        git add Cargo.toml Cargo.lock CHANGELOG.md
        git commit -m "chore(release): ''${tag}"
        git tag -a "$tag" -m "$tag"

        echo "Prepared ''${tag}. Push it with: git push origin HEAD --follow-tags"
      '';
    };

    "verify-release-tag" = {
      description = "Verify a release tag matches Cargo.toml and CHANGELOG.md.";
      exec = ''
        if [ "$#" -ne 1 ]; then
          echo "Usage: verify-release-tag <tag>" >&2
          exit 64
        fi

        tag="''${1#refs/tags/}"
        version="''${tag#v}"
        crate_version="$(
          cargo metadata --no-deps --format-version 1 |
            jq -r '.packages[] | select(.name == "acpx") | .version'
        )"

        if [ "$version" != "$crate_version" ]; then
          echo "Tag (''${tag}) does not match Cargo.toml version (''${crate_version})." >&2
          exit 1
        fi

        if ! grep -Eq "^## \\[''${crate_version//./\\.}\\]" CHANGELOG.md; then
          echo "CHANGELOG.md does not contain an entry for ''${crate_version}." >&2
          exit 1
        fi
      '';
    };
  };

  tasks = {
    "fmt:write" = {
      exec = ''
        cargo fmt --all
        oxfmt ${oxfmtArgs}
      '';
    };

    "fmt:check" = {
      exec = ''
        cargo fmt --all -- --check
        oxfmt --check ${oxfmtArgs}
      '';
    };

    "lint:check" = {
      exec = ''
        typos
        cargo clippy ${cargoAllTargetsAllFeatures} -- -D warnings
      '';
    };

    "lint:fix" = {
      exec = ''
        typos --write-changes
        cargo clippy --fix --all-features --allow-dirty --allow-staged
        typos
        cargo clippy ${cargoAllTargetsAllFeatures} -- -D warnings
      '';
    };

    "test:check" = {
      exec = ''
        cargo nextest run --all ${cargoAllFeaturesLocked} --no-tests pass
        cargo test --doc ${cargoAllFeaturesLocked}
      '';
    };

    "example:check" = {
      exec = ''
        name="$(printf '%s' "$DEVENV_TASK_INPUT" | jq -r '.name // empty')"
        if [ -z "$name" ]; then
          echo "Task input 'name' is required." >&2
          exit 64
        fi

        cargo test --example "$name" ${cargoAllFeaturesLocked}
      '';
    };

    "quality:check" = {
      exec = ''
        cargo fmt --all -- --check
        oxfmt --check ${oxfmtArgs}
        typos
        cargo clippy ${cargoAllTargetsAllFeatures} -- -D warnings
        RUSTDOCFLAGS="-D warnings" cargo doc --no-deps ${cargoAllFeaturesLocked}
        cargo nextest run --all ${cargoAllFeaturesLocked} --no-tests pass
        cargo test --doc ${cargoAllFeaturesLocked}
        cargo test --example cli ${cargoAllFeaturesLocked}
        cargo deny check
        cargo build --all ${cargoAllFeaturesLocked}
      '';
    };

    "quality:fix" = {
      exec = ''
        cargo fmt --all
        oxfmt ${oxfmtArgs}
        typos --write-changes
        cargo clippy --fix --all-features --allow-dirty --allow-staged
        cargo fmt --all -- --check
        oxfmt --check ${oxfmtArgs}
        typos
        cargo clippy ${cargoAllTargetsAllFeatures} -- -D warnings
        RUSTDOCFLAGS="-D warnings" cargo doc --no-deps ${cargoAllFeaturesLocked}
        cargo nextest run --all ${cargoAllFeaturesLocked} --no-tests pass
        cargo test --doc ${cargoAllFeaturesLocked}
        cargo test --example cli ${cargoAllFeaturesLocked}
        cargo deny check
        cargo build --all ${cargoAllFeaturesLocked}
      '';
    };

    "audit:check" = {
      exec = ''
        cargo deny check
      '';
    };

    "build:debug" = {
      exec = ''
        cargo build --all ${cargoAllFeaturesLocked}
      '';
    };

    "doc:build" = {
      exec = ''
        RUSTDOCFLAGS="-D warnings" cargo doc --no-deps ${cargoAllFeaturesLocked}
      '';
    };

    "registry:sync" = {
      exec = ''
        cargo run --bin registry-sync --features registry-sync-bin --
      '';
    };

    "publish:run" = {
      exec = ''
        dry_run="$(printf '%s' "$DEVENV_TASK_INPUT" | jq -r '.dry_run // false')"
        if [ "$dry_run" = "true" ]; then
          cargo publish --locked --dry-run
        else
          cargo publish --locked
        fi
      '';
    };

    "changelog:update" = {
      exec = ''
        git cliff --config cliff.toml --output CHANGELOG.md
      '';
    };

    "release:notes" = {
      exec = ''
        version="$(printf '%s' "$DEVENV_TASK_INPUT" | jq -r '.version // empty')"
        if [ -n "$version" ]; then
          git cliff --config cliff.toml --tag "v$version" --strip header
        else
          git cliff --config cliff.toml --current --strip header
        fi
      '';
    };

    "version:next" = {
      exec = ''
        git cliff --config cliff.toml --bumped-version | sed 's/^v//'
      '';
    };

    "ref:clone" = {
      exec = ''
        url="$(printf '%s' "$DEVENV_TASK_INPUT" | jq -r '.url // empty')"
        name="$(printf '%s' "$DEVENV_TASK_INPUT" | jq -r '.name // empty')"
        if [ -z "$url" ]; then
          echo "Task input 'url' is required." >&2
          exit 64
        fi

        if [ -n "$name" ]; then
          ref-clone "$url" "$name"
        else
          ref-clone "$url"
        fi
      '';
    };

    "ref:copy" = {
      exec = ''
        source_path="$(printf '%s' "$DEVENV_TASK_INPUT" | jq -r '.source // empty')"
        name="$(printf '%s' "$DEVENV_TASK_INPUT" | jq -r '.name // empty')"
        if [ -z "$source_path" ] || [ -z "$name" ]; then
          echo "Task inputs 'source' and 'name' are required." >&2
          exit 64
        fi

        ref-copy "$source_path" "$name"
      '';
    };

    "release:prepare" = {
      exec = ''
        version="$(printf '%s' "$DEVENV_TASK_INPUT" | jq -r '.version // empty')"
        if [ -n "$version" ]; then
          release "$version"
        else
          release
        fi
      '';
    };

    "build:release" = {
      exec = ''
        cargo build --all ${cargoAllFeaturesLocked} --release
      '';
    };

    "msrv:check" = {
      exec = ''
        cargo check ${cargoAllFeaturesLocked}
      '';
    };

    "release:verify-tag" = {
      exec = ''
        tag="$(printf '%s' "$DEVENV_TASK_INPUT" | jq -r '.tag // empty')"
        if [ -z "$tag" ]; then
          echo "Task input 'tag' is required." >&2
          exit 64
        fi

        verify-release-tag "$tag"
      '';
    };

    "release:verify-changelog" = {
      exec = ''
        tmpfile=$(mktemp)
        git cliff --config cliff.toml --output "$tmpfile"
        diff -u CHANGELOG.md "$tmpfile"
      '';
    };

    "release:render-notes" = {
      exec = ''
        output_path="$(printf '%s' "$DEVENV_TASK_INPUT" | jq -r '.output_path // empty')"
        if [ -z "$output_path" ]; then
          echo "Task input 'output_path' is required." >&2
          exit 64
        fi

        git cliff --config cliff.toml --current --strip header --output "$output_path"
      '';
    };

    "release:publish-github" = {
      exec = ''
        if [ -z "''${TAG:-}" ]; then
          echo "TAG must be set." >&2
          exit 64
        fi

        if [ -z "''${RELEASE_NOTES_PATH:-}" ]; then
          echo "RELEASE_NOTES_PATH must be set." >&2
          exit 64
        fi

        if gh release view "$TAG" >/dev/null 2>&1; then
          gh release edit "$TAG" --title "$TAG" --notes-file "$RELEASE_NOTES_PATH"
        else
          gh release create "$TAG" --verify-tag --title "$TAG" --notes-file "$RELEASE_NOTES_PATH"
        fi
      '';
    };

  };

  enterTest = ''
    devenv --no-tui tasks run quality:check
    devenv --no-tui tasks run test:check
  '';

  enterShell = ''
    echo "acpx development shell: use 'just fmt' to format files, 'just quality' for checks, and 'just release [version]' for releases."
  '';
}
