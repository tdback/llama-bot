{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      supportedSystems = [ "x86_64-linux" ];
      eachSystem = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
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
