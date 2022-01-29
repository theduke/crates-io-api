{
  description = "omni-csi";

  inputs = {
    flakeutils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flakeutils }: 
    flakeutils.lib.eachDefaultSystem (system:
      let
        system = "x86_64-linux";
        pkgs = nixpkgs.legacyPackages."${system}";
      in rec {

        devShell = pkgs.stdenv.mkDerivation {
            name = "crates-io-api";
            src = self;
            buildInputs = with pkgs; [
              pkgconfig
              openssl
            ];
            propagatedBuildInputs = with pkgs; [
              openssl
            ];
            buildPhase = "";
            installPhase = "";

            RUST_BACKTRACE = "1";
            LD_LIBRARY_PATH = "${pkgs.openssl.out}/lib";
        };

      }
    );
}  
