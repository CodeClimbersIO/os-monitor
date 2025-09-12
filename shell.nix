{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  # Development dependencies
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer

    # X11 development libraries
    xorg.libX11.dev
    xorg.libXext.dev
    xorg.libXcursor.dev
    xorg.libXi.dev
    xorg.libXrandr.dev

    # Build tools
    pkg-config
    clang
  ];

  # Set up library paths for dynamic linking
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.xorg.libX11
    pkgs.xorg.libXext
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
  ];

  # Welcome message when entering the shell
  shellHook = ''
    echo "🦀 Rust + X11 development environment loaded"
    echo "📦 Available tools: rustc, cargo, clippy, rust-analyzer"
    echo "🔗 X11 libraries configured for linking"
    rustc --version
    cargo --version
  '';
}
