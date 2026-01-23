{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = [
    pkgs.pkg-config
    pkgs.webkitgtk_4_1
    pkgs.wayland
    pkgs.libxkbcommon
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
