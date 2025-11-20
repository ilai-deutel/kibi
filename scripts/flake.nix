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
      bash_script =
        name:
        { ... }@args:
        pkgs.writeShellApplication (
          {
            name = "${name}.sh";
            text = (builtins.readFile ./${name}.sh);
            bashOptions = [ ]; # Set in script
          }
          // args
        );
    in
    {
      packages.x86_64-linux = (
        builtins.mapAttrs bash_script {
          prepare_release = {
            runtimeInputs = with pkgs; [
              b2sum
              b3sum
              gh
              jq
              libxml2
              python3Packages.ed25519-blake2b
              python3Packages.mdformat
              python3Packages.mdformat-gfm
              slsa-verifier
              rustup
            ];
          };
          generate_recording = {
            runtimeInputs = with pkgs; [
              nodePackages.svgo
              (pkgs.rWrapper.override {
                packages = with rPackages; [
                  asciicast
                  httr2
                  openssl
                  xml2
                ];
              })
            ];
            excludeShellChecks = [ "SC1091" ];
          };
        }
      );
    };
}
