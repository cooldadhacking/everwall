{ pkgs, lib, ... }:
{
  # Rust — edition 2024 requires rustc >= 1.85
  languages.rust.enable = true;
  languages.rust.channel = "stable";
  # default components: cargo, rustc, clippy, rustfmt, rust-analyzer

  packages = with pkgs; [
    cargo-nextest
    pkg-config
  ] ++ lib.optionals pkgs.stdenv.isDarwin [ libiconv ];
}
