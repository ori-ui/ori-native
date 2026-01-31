{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = [
    pkgs.pkg-config
    pkgs.gtk4
    pkgs.librsvg
    pkgs.gtk4-layer-shell
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
