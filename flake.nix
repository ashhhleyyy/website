{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
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

        craneLib = crane.mkLib pkgs;

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

        common-args = {
          inherit src buildInputs;
          strictDeps = true;
        };

        individualCrateArgs = common-args // {
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type: (craneLib.filterCargoSources path type)
              || (builtins.match ".*html$" path != null)
              || (builtins.match ".*/assets/images/pfp\\.png$" path != null)
              || (builtins.match ".*/(blog|projects)/.*\\.md$" path != null);
            name = "source";
          };
        };

        assetwrapCargoArgs = "-p assetwrap";
        assetwrapInternalCargoArgs = "${assetwrapCargoArgs} --no-default-features";

        websiteCargoArtifacts = craneLib.buildDepsOnly common-args;
        assetwrapInternalCargoArtifacts = craneLib.buildDepsOnly (common-args // {
          cargoExtraArgs = assetwrapInternalCargoArgs;
        });
        assetwrapCargoArtifacts = craneLib.buildDepsOnly (common-args // {
          cargoExtraArgs = assetwrapCargoArgs;
        });

        assetwrap-internal = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "assetwrap";
            cargoArtifacts = assetwrapInternalCargoArtifacts;
            cargoExtraArgs = "-p assetwrap --no-default-features";
          }
        );

        assetwrap = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "assetwrap";
            cargoArtifacts = assetwrapCargoArtifacts;
            cargoExtraArgs = "-p assetwrap";
          }
        );

        assetindex = pkgs.runCommand "assetindex.json" {} ''
        export ASSET_INDEX_OUT_PATH=$out
        mkdir assets
        cp -r ${./assets}/* ./assets/
        ls -lah
        ${assetwrap-internal}/bin/assetwrap ${./assetconfig.json}
        '';

        ASSET_INDEX = assetindex;

        website = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "website";
            cargoArtifacts = websiteCargoArtifacts;
            cargoExtraArgs = "-p website";
            inherit ASSET_INDEX;
          }
        );

        rust-toolchain = pkgs.rust-bin.stable.latest.default;
        rust-dev-toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
    {
      checks = {
        # Build the crate as part of `nix flake check` for convenience
        inherit assetwrap website;

        website-clippy = craneLib.cargoClippy {
          inherit src buildInputs;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          cargoArtifacts = websiteCargoArtifacts;
          inherit ASSET_INDEX;
        };

        # Check formatting
        website-fmt = craneLib.cargoFmt {
          inherit src;
          inherit ASSET_INDEX;
        };
      };

      packages = {
        inherit website assetwrap assetwrap-internal assetindex;
        default = website;
        docker.website = pkgs.callPackage ./docker.nix {
          inherit website;
        };
      };

      apps = let
        website = flake-utils.lib.mkApp {
          drv = website;
        };
        assetwrap = flake-utils.lib.mkApp {
          drv = assetwrap;
        };
      in
      {
        inherit website assetwrap;
        default = website;
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
