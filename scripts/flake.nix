{
  description = "CI scripts";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/25.05";
  };
  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      script =
        name:
        { ... }@args:
        pkgs.writeShellApplication {
          name = "${name}.sh";
          text = (builtins.readFile ./${name}.sh);
          bashOptions = [ ]; # Set in script
        }
        // args;
    in
    {
      packages.x86_64-linux = builtins.mapAttrs script {
        prepare_release = {
          runtimeInputs = with pkgs; [
            gh
            jq
            slsa-verifier
            rustup
            python313Packages.mdformat
            python313Packages.mdformat-gfm
          ];
        };
      };
    };
}
