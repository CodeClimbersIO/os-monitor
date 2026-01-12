{
  description = "OS Monitor - Cross-platform system monitoring";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      #
      # === VM for Testing ===
      # Run with: nix run .#vm-x11
      # Or use the `svm` alias in the dev shell
      #
      vmConfigurations = {
        x11 = nixpkgs.lib.nixosSystem {
          system = "x86_64-linux";
          modules = [
            (
              { pkgs, modulesPath, ... }:
              let
                # Build os-monitor from the current source
                os-monitor = pkgs.rustPlatform.buildRustPackage {
                  pname = "os-monitor";
                  version = "0.4.9";
                  src = ./.;

                  cargoLock = {
                    lockFile = ./Cargo.lock;
                  };

                  nativeBuildInputs = with pkgs; [
                    pkg-config
                    clang
                  ];

                  buildInputs = with pkgs; [
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
                };
              in
              {
                imports = [ (modulesPath + "/virtualisation/qemu-vm.nix") ];

                # VM settings
                virtualisation = {
                  memorySize = 4096; # 4GB RAM
                  cores = 4;
                  diskSize = 20480; # 20GB disk
                  forwardPorts = [
                    { from = "host"; host.port = 2222; guest.port = 22; }
                  ];
                };

                # Use X11 with XFCE (lighter than GNOME, more reliable in VMs)
                services.xserver.enable = true;
                services.xserver.displayManager.lightdm.enable = true;
                services.xserver.desktopManager.xfce.enable = true;

                # Basic packages for testing
                environment.systemPackages = with pkgs; [
                  vim
                  git
                  firefox
                  mousepad # Simple text editor for blocking tests
                  gnome-calculator # Simple calculator for multi-app blocking tests
                  os-monitor # Our monitoring app
                ];

                # Auto-start os-monitor as a system service (needs root for /dev/input)
                systemd.services.os-monitor = {
                  description = "OS Monitor - Activity and blocking service";
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" ];
                  serviceConfig = {
                    ExecStart = "${os-monitor}/bin/os-monitor";
                    Restart = "always";
                    RestartSec = "5s";
                    Environment = "RUST_LOG=info";
                    # Run as root (required for /dev/input access)
                    User = "root";
                  };
                };

                # Test user (password: test)
                users.users.test = {
                  isNormalUser = true;
                  extraGroups = [
                    "wheel"
                    "networkmanager"
                  ];
                  initialPassword = "test";
                };

                # Allow test user to sudo without password (for convenience)
                security.sudo.wheelNeedsPassword = false;

                # Auto-login for convenience
                services.displayManager.autoLogin = {
                  enable = true;
                  user = "test";
                };

                # Networking
                networking = {
                  hostName = "nixos-x11-vm";
                  networkmanager.enable = true;
                };

                # SSH for backup access
                services.openssh.enable = true;

                # Enable sound
                services.pulseaudio.enable = false;
                services.pipewire = {
                  enable = true;
                  alsa.enable = true;
                  pulse.enable = true;
                };

                system.stateVersion = "25.11";
              }
            )
          ];
        };
      };

      # VM package
      packages.x86_64-linux = {
        vm-x11 = self.vmConfigurations.x11.config.system.build.vm;
      };

      #
      # === Dev Shell ===
      # Enter with: nix develop (or direnv allow)
      #
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          isLinux = pkgs.stdenv.isLinux;
          isDarwin = pkgs.stdenv.isDarwin;
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              pkg-config
              clang
            ];

            buildInputs =
              with pkgs;
              [
                rustc
                cargo
                rustfmt
                clippy
                rust-analyzer
              ]
              ++ pkgs.lib.optionals isLinux [
                # X11 libs only needed on Linux
                xorg.libX11
                xorg.libXext
                xorg.libXcursor
                xorg.libXi
                xorg.libXrandr
              ]
              ++ pkgs.lib.optionals isDarwin [
                # macOS frameworks
                darwin.apple_sdk.frameworks.Cocoa
                darwin.apple_sdk.frameworks.Security
              ];

            LIBRARY_PATH = pkgs.lib.optionalString isLinux (
              pkgs.lib.makeLibraryPath [
                pkgs.xorg.libX11
                pkgs.xorg.libXext
                pkgs.xorg.libXcursor
                pkgs.xorg.libXi
                pkgs.xorg.libXrandr
              ]
            );

            shellHook = ''
              echo "Rust development environment loaded (${system})"
            '' + pkgs.lib.optionalString isDarwin ''
              echo "Note: VM testing requires Linux. Use a remote Linux machine or cloud VM."
            '';
          };
        }
      );
    };
}
