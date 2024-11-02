{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/24.05";

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
          channel = "1.76";
          date = "2024-02-08";
          sha256 = "e4mlaJehWBymYxJGgnbuCObVlqMlQSilZ8FljG9zPHY=";
        };

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain.rust;
          rustc = toolchain.rust;
        };

        cargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        name = cargo.package.name;
        version = cargo.package.version;

        lintingRustFlags = "-D unused-crate-dependencies";
      in {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            # Rust toolchain
            toolchain.toolchain

            # Code formatting tools
            alejandra
            treefmt

            # Dependency audit tool
            cargo-deny

            # Container image management
            skopeo
          ];

          RUSTFLAGS = lintingRustFlags;
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
        };
      }
    );
}
