{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    clang
  ];

  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer

    xorg.libX11
    xorg.libXext
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];

  LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.xorg.libX11
    pkgs.xorg.libXext
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
  ];

  shellHook = ''
    echo "Rust + X11 development environment loaded"
  '';
}
