{
  description = "Islamic Prayer Times Information and Notifications";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        buildInputs = with pkgs; [
          openssl
          dbus
        ];
      in
      {
        packages = {
          prayer-times = pkgs.rustPlatform.buildRustPackage {
            pname = "prayer-times";
            version = "0.4.0";

            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;

            inherit nativeBuildInputs buildInputs;

            postInstall = ''
              install -Dm644 assets/mosque-svgrepo-com.png $out/share/icons/hicolor/128x128/apps/prayer-times.png
            '';

            meta = with pkgs.lib; {
              description = "Islamic Prayer Times Information and Notifications";
              homepage = "https://github.com/Yasso9/prayer-times";
              license = licenses.mit;
              mainProgram = "prayer-times";
            };
          };

          default = self.packages.${system}.prayer-times;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.prayer-times ];

          packages = with pkgs; [
            rust-analyzer
            clippy
            rustfmt
          ];
        };
      }
    )
    // {
      overlays.default = _final: prev: {
        prayer-times = self.packages.${prev.system}.prayer-times;
      };
    };
}
