{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "kak-tree-sitter";
  version = "0.0.0";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
