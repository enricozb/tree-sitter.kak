{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "kak-tree-sitter";
  version = "0.1.8";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
