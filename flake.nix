# this is a WIP flake for development and experimentation only
{
  description = "dantleech/pttlog";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = inputs @ {
    self,
    flake-parts,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux"];

      perSystem = {
        pkgs,
        system,
        ...
      }: {
        formatter = pkgs.alejandra;
        devShells.default = pkgs.mkShellNoCC {
          name = "dev";

          buildInputs = [
            pkgs.rustup
            pkgs.gcc
          ];
        };
      };
    };
}
