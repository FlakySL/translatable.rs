{
  description = "Flake configuration file for translatable.rs development.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      fenix,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        crane = inputs.crane.mkLib pkgs;

        toolchain =
          with fenix.packages.${system};
          combine [
            minimal.rustc
            minimal.cargo
            complete.rust-src
            complete.rustfmt
            complete.clippy
          ];

        craneLib = crane.overrideToolchain toolchain;
      in
      {
        devShells.default = craneLib.devShell {
          packages = with pkgs; [
            toolchain
            rustfmt
            clippy
            qemu-user
          ];

          env = {
            LAZYVIM_RUST_DIAGNOSTICS = "bacon-ls";
          };
        };
      }
    );
}
