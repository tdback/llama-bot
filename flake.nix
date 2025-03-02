{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      supportedSystems = [
        "aarch64-linux"
        "x86_64-linux"
      ];
      eachSystem = nixpkgs.lib.genAttrs supportedSystems;
      forPkgs =
        fn:
        nixpkgs.lib.mapAttrs (system: pkgs: (fn pkgs)) (
          nixpkgs.lib.getAttrs supportedSystems nixpkgs.legacyPackages
        );
    in
      {
      packages = forPkgs (pkgs: {
        default = pkgs.callPackage ./nix { };
      });

      devShells = eachSystem (
        system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              openssl
              pkg-config
              rust-analyzer
              rust-bin.stable.latest.default
            ];
          };
        }
      );
    };
}
