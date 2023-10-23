{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";

    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        toolchain = fenix.packages.${system}.toolchainOf {
          channel = "1.72";
          date = "2023-09-19";
          sha256 = "dxE7lmCFWlq0nl/wKcmYvpP9zqQbBitAQgZ1zx9Ooik=";
        };

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain.rust;
          rustc = toolchain.rust;
        };

        cargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        name = cargo.package.name;
        version = cargo.package.version;
      in {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            toolchain.toolchain

            alejandra
            treefmt

            cargo-deny
          ];
        };

        packages = rec {
          default = naersk'.buildPackage {
            name = name;
            version = version;

            src = ./.;
          };

          container-image = pkgs.dockerTools.buildImage {
            name = "cloudflare-ddns-updater";
            tag = "latest";
            created = "now";

            copyToRoot = pkgs.buildEnv {
              name = "image-root";
              paths = [pkgs.bashInteractive pkgs.coreutils];
              pathsToLink = ["/bin"];
            };

            config = {
              Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${default}/bin/cloudflare-ddns-updater"];
              Env = [
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              ];
            };
          };

          clippy = naersk'.buildPackage {
            src = ./.;
            mode = "clippy";
          };

          test = naersk'.buildPackage {
            src = ./.;
            mode = "test";

            # Ensure detailed test output appears in nix build log
            cargoTestOptions = x: x ++ ["1>&2"];
          };
        };
      }
    );
}
