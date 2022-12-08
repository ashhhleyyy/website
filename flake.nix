{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        inherit (pkgs) lib;

        craneLib = crane.lib.${system};

        src = craneLib.cleanCargoSource ./.;

        deps = [
          pkgs.bintools
          pkgs.nasm
          pkgs.openssl
          pkgs.pkg-config
        ] ++ lib.optionals pkgs.stdenv.isDarwin [
          # Additional darwin specific inputs can be set here
          pkgs.libiconv
        ];

        buildInputs = [
          rust-toolchain
        ] ++ deps;

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src buildInputs;
        };

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        website = craneLib.buildPackage {
          inherit cargoArtifacts src buildInputs;
        };

        rust-toolchain = pkgs.rust-bin.stable.latest.default;
        rust-dev-toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
    {
      checks = {
        # Build the crate as part of `nix flake check` for convenience
        inherit website;

        # Run clippy (and deny all warnings) on the crate source,
        # again, resuing the dependency artifacts from above.
        #
        # Note that this is done as a separate derivation so that
        # we can block the CI if there are issues here, but not
        # prevent downstream consumers from building our crate by itself.
        website-clippy = craneLib.cargoClippy {
          inherit cargoArtifacts src buildInputs;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        };

        # Check formatting
        website-fmt = craneLib.cargoFmt {
          inherit src;
        };
      };

      packages.default = website;

      apps.default = flake-utils.lib.mkApp {
        drv = website;
      };

      devShells.default = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.checks;

        # Extra inputs can be added here
        nativeBuildInputs = with pkgs; [
          rust-dev-toolchain
        ] ++ deps;
      };
    });
}
