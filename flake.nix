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
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      #
      # === VM for Testing ===
      # Run with: nix run .#vm-gnome-x11
      # Or use the `svm` alias in the dev shell
      #
      vmConfigurations = {
        gnome-x11 = nixpkgs.lib.nixosSystem {
          system = "x86_64-linux";
          modules = [
            (
              { config, pkgs, lib, modulesPath, ... }:
              {
                imports = [ (modulesPath + "/virtualisation/qemu-vm.nix") ];

                # VM settings
                virtualisation = {
                  memorySize = 4096; # 4GB RAM
                  cores = 4;
                  diskSize = 20480; # 20GB disk
                  qemu.options = [
                    "-device virtio-vga-gl"
                    "-display gtk,gl=on"
                  ];
                };

                # Use X11 (not Wayland)
                services.xserver.enable = true;
                services.displayManager.gdm = {
                  enable = true;
                  wayland = false; # Force X11
                };
                services.desktopManager.gnome.enable = true;

                # Exclude some heavy GNOME apps to speed up build
                environment.gnome.excludePackages = with pkgs; [
                  gnome-tour
                  epiphany # web browser
                  geary # email
                ];

                # Basic packages for testing
                environment.systemPackages = with pkgs; [
                  vim
                  git
                  firefox
                ];

                # Test user (password: test)
                users.users.test = {
                  isNormalUser = true;
                  extraGroups = [ "wheel" "networkmanager" ];
                  initialPassword = "test";
                };

                # Allow test user to sudo without password (for convenience)
                security.sudo.wheelNeedsPassword = false;

                # Auto-login for convenience (optional, comment out if you prefer login screen)
                services.displayManager.autoLogin = {
                  enable = true;
                  user = "test";
                };

                # Networking
                networking = {
                  hostName = "nixos-gnome-vm";
                  networkmanager.enable = true;
                };

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
        vm-gnome-x11 = self.vmConfigurations.gnome-x11.config.system.build.vm;
      };

      #
      # === Dev Shell ===
      # Enter with: nix develop (or direnv allow)
      #
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
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

              # Start the GNOME X11 test VM
              svm() {
                echo "Starting GNOME X11 VM..."
                nix run .#vm-gnome-x11
              }
            '';
          };
        }
      );
    };
}
