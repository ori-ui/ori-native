{ pkgs ? import <nixpkgs> {
  config = {
    allowUnfree = true;
    android_sdk.accept_license = true;
  };
} }:

let
  androidComposition = pkgs.androidenv.composeAndroidPackages {
    buildToolsVersions = [ "37.0.0" "36.0.0" "35.0.0" ];
    platformVersions = [ "37" "36" "35" ];
    abiVersions = [ "armeabi-v7a" "arm64-v8a" ];
    ndkVersions = [ "27.3.13750724" ];
    includeNDK = true;
  };
  androidSdk = androidComposition.androidsdk;
in  pkgs.mkShell rec {
  buildInputs = with pkgs; [
    pkg-config
    gtk4
    gtk4-layer-shell
    librsvg

    androidSdk
    gradle_9
  ];

  ANDROID_SDK_ROOT = "${androidSdk}/libexec/android-sdk";
  GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidSdk}/libexec/android-sdk/build-tools/37.0.0/aapt2";

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
