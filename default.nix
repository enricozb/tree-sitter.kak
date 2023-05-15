{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "kak-sitter";
  version = "0.1.10";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
