{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          packages = [
            just
            cargo-msrv
            git-cliff
          ];

          nativeBuildInputs = [
            (lib.hiPrio rust-bin.nightly."2025-04-30".rustfmt)
            rust-bin.stable.latest.default
          ];

          buildInputs = [ ];
        };
      }
    );
}
