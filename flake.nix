{
  inputs = {
    nixpkgs.url = github:nixos/nixpkgs/nixos-21.11;
    utils.url = github:numtide/flake-utils;
  };

  outputs = { self, nixpkgs, utils, mach-nix, ... } @ inputs:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        jdtls = pkgs.callPackage ./jdt-language-server.nix {};
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            openjdk8
            maven
            jdtls
          ];
        };
      });
}
