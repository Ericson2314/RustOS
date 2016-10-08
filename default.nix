{ stdenv, callPackage, fetchFromGitHub, runCommand, makeWrapper, pkgsi686Linux
, gdb, nasm, qemu, bochs, valgrind
, release ? true
}:

let
  rust-nightly-nix = fetchFromGitHub {
    owner = "solson";
    repo = "rust-nightly-nix";
    rev = "0021290418184a495ee8f9b00bf8e33a3463b653";
    sha256 = "0zh4153q0rn9kg2ssjddcxgckxf0ak09bcbrxx06n14ad7726gp4";
  };

  funs = callPackage rust-nightly-nix { };

  cargoNightly = funs.cargo {
    #date = "2016-09-09";
  };

  #rustDate = "2016-09-09";

  rustcNightly = funs.rustc {
    #date = rustDate;
  };

  rustNightlyWithi686 = funs.rustcWithSysroots {
    rustc = rustcNightly;
    sysroots = [
      (funs.rust-std {
        #date = rustDate;
      })
      (funs.rust-std {
        #date = rustDate;
        system = "x86_64-unknown-linux-gnu";
      })
      (funs.rust-std {
        #date = rustDate;
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
