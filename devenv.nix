{ pkgs ? import <nixpkgs>, lib, ... }:




# let
#   uniffi-bindgen-cs = pkgs.rustPlatform.buildRustPackage rec {
#     name = "uniffi-bindgen-cs-${version}";
#     version = "0.8.0+v0.25.0";

#     src = pkgs.fetchFromGitHub {
#       owner = "NordSecurity";
#       repo = "uniffi-bindgen-cs";
#       rev = "v${version}";
#       sha256 = "sha256:1ixvl6lkjmds7gyvhfc4dibjy8g4c5vr4ra32flkvns0f3h60bd0";
#       fetchSubmodules = true;
#     };

#     cargoSha256 = "sha256:1ixvl6lkjmds7gyvhfc4dibjy8g4c5vr4ra32flkvns0f3h60bd0";

#     nativeBuildInputs = with pkgs; [ pkg-config ];
#     buildInputs = with pkgs; [ openssl ];
#   };
# in
{
  name = "dnssec-prover";

  packages = [
    pkgs.bash
    pkgs.git
    pkgs.rustup
    pkgs.nix-prefetch-git
    pkgs.wasm-pack
    # uniffi-bindgen-cs
  ] ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [
    frameworks.Security
  ]);

  languages = {
    rust = let 
      rustVersion = pkgs.rust-bin.stable.latest.default;
    in {
      enable = true;
      toolchain.rustc = (rustVersion.override {
        extensions = [ "rust-src" ];
        targets = [ "wasm32-unknown-unknown" "wasm32-wasi"];
      });
    };
  };

  # macOS workaround:
  # The linker on macOS doesn't like the frameworks option when compiling to wasm32.
  # See https://github.com/rust-lang/rust/issues/122333
  env.RUSTFLAGS = lib.mkIf pkgs.stdenv.isDarwin (lib.mkForce "");

  # FIXME: This is a workaround for the fact that the uniffi-bindgen-cs package is not available in nixpkgs
  enterShell = ''
    if ! command -v uniffi-bindgen-cs &> /dev/null
    then
        cargo install uniffi-bindgen-cs --git https://github.com/NordSecurity/uniffi-bindgen-cs --tag v0.8.0+v0.25.0
    fi
  '';
}