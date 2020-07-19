{ pkgs ? import <nixpkgs> {} }:

pkgs.stdenv.mkDerivation {
  name = "cratesioapi";
  buildInputs = with pkgs; [
      pkgconfig
      lld_9
      openssl
  ];

  shellHook = ''
    # Use lld as a linker.
    export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
    export LD_LIBRARY_PATH="${pkgs.openssl.out}/lib:$LD_LIBRARY_PATH"
  '';
}
