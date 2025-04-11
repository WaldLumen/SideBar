{
  description = "Flake для Rust-проєкту SideBar";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system;
          overlays = overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.pkg-config
            pkgs.openssl
            pkgs.gtk3      # для eframe/egui, якщо GUI
            pkgs.cairo
            pkgs.glib
	    pkgs.libxkbcommon
          ];

          RUSTFLAGS = "--cfg tokio_unstable"; # іноді потрібно для tokio з "full"
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "SideBar";
          version = "1.0.1";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = [ pkgs.openssl pkgs.pkg-config ];

          nativeBuildInputs = [ pkgs.pkg-config ];

          # Вказуємо на локальну залежність
          # Важливо, щоб CaloryFetch був доступний у дереві flake
          postPatch = ''
            ln -s ${../CaloryFetch} calory_fetch
          '';
        };
      }
    );
}
