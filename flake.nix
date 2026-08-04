{
  description = "Ryp - A text editor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        nerdFontTar = pkgs.fetchurl {
          url = "https://github.com/ryanoasis/nerd-fonts/releases/download/v3.3.0/0xProto.tar.xz";
          hash = "sha256-1yvdg/AqkHHUuwrNOKN/5oeke9KRieg47fhH7Tm4fhA=";
        };
        cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.package.version;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          postPatch = ''
            perl -0777 -pi -e '
              s/let response = ureq::get\(url\)\s*\.call\(\)\s*\.expect\([^;]*\);/
              let font_tar_path = std::env::var("RYP_FONT_TAR").expect("RYP_FONT_TAR not set");/s
            ' build.rs

            perl -0777 -pi -e '
              s/let mut body = response\.into_body\(\);\s*let xz_reader = xz2::read::XzDecoder::new\(body\.as_reader\(\)\);/
              let font_tar_file = std::fs::File::open(\&font_tar_path).expect("Failed to open vendored font tarball");
              let xz_reader = xz2::read::XzDecoder::new(font_tar_file);/s
            ' build.rs
          '';
          RYP_FONT_TAR = nerdFontTar;
          pname = cargoToml.package.name;
          inherit version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = [
            pkgs.openssl
            pkgs.zlib
            pkgs.libssh2
          ];

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.perl
          ];

          meta = with pkgs.lib; {
            description = cargoToml.package.description or "Rust application";
            license = licenses.bsd2;
            platforms = platforms.unix;
            ryp = cargoToml.package.name;
          };
        };

        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.rustc
            pkgs.cargo
            pkgs.rustfmt
            pkgs.clippy
            pkgs.rust-analyzer
            pkgs.pkg-config

            pkgs.openssl
            pkgs.zlib
            pkgs.libssh2
          ];
        };
      }
    )
    // {
      overlays.default = final: prev: {
        ryp = self.packages.${final.system}.default;
      };
    };
}
