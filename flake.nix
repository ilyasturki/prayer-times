{
  description = "Islamic Prayer Times Information and Notifications";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          prayer-times = pkgs.rustPlatform.buildRustPackage {
            pname = "prayer-times";
            version = cargoToml.package.version;

            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [
              openssl
              dbus
            ];

            # config::tests::test_config_from_str asserts a debug-mode asset
            # path; it only passes in debug builds.
            checkFlags = [ "--skip=config::tests::test_config_from_str" ];

            postInstall = ''
              install -Dm644 assets/mosque-svgrepo-com.png \
                $out/share/icons/hicolor/128x128/apps/prayer-times.png

              # generate-shell writes to env!(CARGO_MANIFEST_DIR)/target/completions
              # which is the build directory at compile time (still writable here).
              $out/bin/prayer-times generate-shell
              completions=$(find . -type d -name completions -path '*/target/*' | head -n1)
              install -Dm644 "$completions/prayer-times.bash" \
                "$out/share/bash-completion/completions/prayer-times"
              install -Dm644 "$completions/_prayer-times" \
                "$out/share/zsh/site-functions/_prayer-times"
              install -Dm644 "$completions/prayer-times.fish" \
                "$out/share/fish/vendor_completions.d/prayer-times.fish"
            '';

            meta = with pkgs.lib; {
              description = cargoToml.package.description;
              homepage = cargoToml.package.homepage;
              license = licenses.mit;
              mainProgram = "prayer-times";
              platforms = supportedSystems;
            };
          };
        in
        {
          inherit prayer-times;
          default = prayer-times;
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.prayer-times ];
            packages = with pkgs; [
              rust-analyzer
              clippy
              rustfmt
            ];
          };
        }
      );

      overlays.default = _final: prev: {
        prayer-times = self.packages.${prev.system}.prayer-times;
      };
    };
}
