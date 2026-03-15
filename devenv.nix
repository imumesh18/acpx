{
  pkgs,
  lib,
  config,
  ...
}:
{
  # https://devenv.sh/languages/
  languages.rust.enable = true;
  packages = with pkgs; [
    cargo-deny
    cargo-nextest
    gh
    git
    git-cliff
    jq
    just
    rsync
    rustup
    typos
  ];

  enterShell = ''
    echo "acpx development shell: use 'just ci' for checks and 'just release [version]' for releases."
  '';

  enterTest = ''
    just ci
  '';
}
