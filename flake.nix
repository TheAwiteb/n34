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
        # n34 development environment
        devShells.default = mkShell {
          packages = [
            git-cliff
            mdbook
            nushell
            taplo
          ];

          nativeBuildInputs = [
            (lib.hiPrio rust-bin.nightly."2025-08-07".rustfmt)
            rust-bin.stable.latest.default
            rust-analyzer
          ];
        };

        # Build n34
        packages.default =
          let
            manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
          in
          with pkgs;
          rustPlatform.buildRustPackage {
            pname = manifest.name;
            version = manifest.version;
            cargoLock.lockFile = ./Cargo.lock;
            src = lib.cleanSource ./.;

            meta = {
              inherit (manifest) description homepage;
              license = lib.licenses.gpl3Plus;
            };
          };
      }
    );
}
