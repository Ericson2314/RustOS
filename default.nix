{ callPackage, stdenv, gdb, nasm, qemu, valgrind, rustcNightly, fetchFromGitHub }:

let
  settings = fetchFromGitHub {
    owner = "Ericson2314";
    repo = "nixos-configuration";
    rev = "036ef0cfefacc2b292a29f3ef0a1fac9c339f6b0";
    sha256 = "05n26kq3595j2q2wga20w3zh9cy2dnwmrni3jzggzbxln53cmd8w";
  };

  funs = callPackage "${settings}/user/.nixpkgs/rust-nightly.nix" { };

  rustcNightly = funs.rustc {
    date = "2015-09-01";
    hash = "1a5nn8iwivb0ay3xkb7yg5nsii1zhjw66bz0a21yy8lvnyf4177d";
  };

  cargoNightly = funs.cargo {
    date = "2015-08-20";
    hash = "16lb1ximivzp0v1afmv2538w6wvkln0wg0429lpg97n0j5rapi1i";
  };

in stdenv.mkDerivation {
  name = "RustOS";

  nativeBuildInputs = [ gdb nasm qemu valgrind rustcNightly cargoNightly ];
  src = ./.;

  enableParallelBuilding = true;
}
