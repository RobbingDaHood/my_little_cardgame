# Use the following command to start up: nix develop
# Remember to start idea from that shell to have all default values. 
# By this command: tmux new-session -d "idea-ultimate"
# Cant start multiple IDEAS in parallel yet: https://stackoverflow.com/questions/5889413/start-two-instances-of-intellij-ide
# But maybe that is not a problem. 
# Idea rust toolchain usually is set on its own, but the standard library is maybe not set. Below is standard paths:
# Toolchoin: /nix/store/3vxnv3w6y68ad87sddywh1y7hai3a6gc-rust-default-1.79.0-nightly-2024-03-28/bin
# Standard lib: /nix/store/3vxnv3w6y68ad87sddywh1y7hai3a6gc-rust-default-1.79.0-nightly-2024-03-28/lib/rustlib/src/rust
# Can be find by using: which cargo



# flake.nix
# https://ayats.org/blog/nix-rustup/
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };
    toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        toolchain
      ];
      shellHook = ''
        $SHELL
      '';
      RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
    };
  };
}
