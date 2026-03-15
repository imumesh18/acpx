{
  pkgs,
  ...
}:
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
    jq
    just
    rsync
    typos
  ];

  enterShell = ''
    echo "acpx development shell: use 'just fmt' to format files, 'just ci' for checks, and 'just release [version]' for releases."
  '';

  enterTest = ''
    just ci
  '';
}
