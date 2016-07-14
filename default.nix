{ stdenv, callPackage, fetchFromGitHub, runCommand, makeWrapper, pkgsi686Linux
, gdb, nasm, qemu, bochs, valgrind
, release ? true
}:

let
  settings = fetchFromGitHub {
    owner = "Ericson2314";
    repo = "nixos-configuration";
    rev = "7a24ff14977cf0aee7c62b391233e1b4a892ff3a";
    sha256 = "08z0lwnfqm653i40bdq54pzjryr45lb17ckdjw8hz9s07xvwpx09";
  };

  funs = callPackage "${settings}/user/.nixpkgs/rust-nightly.nix" { };

  cargoNightly = funs.cargo {
    date = "2016-07-12";
  };

  rustDate = "2016-07-12";

  rustcNightly = funs.rustc {
    date = rustDate;
  };

  rustNightlyWithi686 = funs.rustcWithSysroots {
    rustc = rustcNightly;
    sysroots = [
      (funs.rust-std {
        date = rustDate;
      })
      (funs.rust-std {
        date = rustDate;
        system = "i686-linux";
      })
    ];
  };

  cc32 = runCommand "mk-cc32" {
    buildInputs = [ makeWrapper ];
  } ''
    mkdir -p $out/bin
    makeWrapper ${pkgsi686Linux.stdenv.cc}/bin/cc $out/bin/cc32
  '';

in stdenv.mkDerivation {
  name = "RustOS";

  nativeBuildInputs = [
    cc32 gdb nasm qemu bochs valgrind cargoNightly
    # We don't actually need prebuild std libs, but they're nice for testing libraries we
    # use, etc.
    (if release then rustcNightly else rustNightlyWithi686)
  ] ++ stdenv.lib.optionals (!release) [
    rustcNightly.doc
  ];

  src = ./.;

  enableParallelBuilding = true;
}
