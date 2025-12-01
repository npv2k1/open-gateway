{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy

    # Build dependencies
    pkg-config
    openssl
    sqlite

    # Additional development tools
    git
  ] ++ lib.optionals stdenv.isDarwin [
    # macOS specific dependencies
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  shellHook = ''
    echo "🦀 Rust development environment (shell.nix)"
    echo "Rust version: $(rustc --version)"
    echo ""
    echo "To use flakes instead, run: nix develop"
    echo ""
  '';

  # Environment variables
  RUST_BACKTRACE = "1";
  DATABASE_URL = "todo.db";
}
