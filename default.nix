{ stdenv, callPackage, fetchFromGitHub, runCommand, makeWrapper, pkgsi686Linux
, gdb, nasm, qemu, bochs, valgrind
, release ? true
}:

let
  settings = fetchFromGitHub {
    owner = "Ericson2314";
    repo = "nixos-configuration";
    rev = "9da64ec7e950168a6e8bab53ef0ba8bd92781f6b";
    sha256 = "0vznn90b38yx62p1mv8vr2qpzw50cryabm0fz90cd4l8v4n20n8a";
  };

  funs = callPackage "${settings}/user/.nixpkgs/rust-nightly.nix" { };

  cargoNightly = funs.cargo {
    date = "2016-07-17";
  };

  rustDate = "2016-07-17";

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
        system = "x86_64-unknown-linux-gnu";
      })
      (funs.rust-std {
        date = rustDate;
        system = "i686-unknown-linux-gnu";
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
