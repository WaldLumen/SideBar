
{ pkgs ? import <nixpkgs> { overlays = [ (import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz)) ]; } }:

with pkgs;

mkShell {
  nativeBuildInputs = with xorg; [
    libxcb
    libXcursor
    libXrandr
    libXi
    pkg-config
  ] ++ [
    python3
    libGL
    libGLU
  ];
  buildInputs = [
  latest.rustChannels.stable.rust
  xorg.libX11
  wayland
  libxkbcommonFull

];

shellHook = ''

  export LD_LIBRARY_PATH=${lib.makeLibraryPath [libxkbcommonFull]}:$LD_LIBRARY_PATH
'';

}