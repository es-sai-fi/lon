{
  system ? builtins.currentSystem,
}:
let
  sources = import ./lon.nix;
  pkgs = import sources.nixpkgs { inherit system; };
  lib = pkgs.lib;
in
rec {
  packages = import ./nix/packages { inherit pkgs; };

  checks = {
    pre-commit = import ./nix/pre-commit.nix;

    tests = lib.recurseIntoAttrs (
      import ./nix/tests {
        inherit pkgs;
        extraBaseModules = {
          lon-tests = {
            environment.systemPackages = [ packages.lonTests ];
          };
        };
      }
    );
  };
}
