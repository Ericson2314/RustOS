{ stdenv, callPackage, fetchFromGitHub, runCommand, makeWrapper, pkgsi686Linux
, gdb, nasm, qemu, valgrind
, release ? true
}:

let
  settings = fetchFromGitHub {
    owner = "Ericson2314";
    repo = "nixos-configuration";
    rev = "e58b956890aad567c22cfa825036580653b81030";
    sha256 = "1x99wf4my91vcfybq8x8qzs44fv82cpyhafhnf3qss2lxpsqn2lz";
  };

  funs = callPackage "${settings}/user/.nixpkgs/rust-nightly.nix" { };

  rustcNightly = funs.rustc {
    date = "2016-04-20";
    hash = "0mwkaldhcqdy297hjggvv1gbmxhi7581c5byk3z404p3ic063zlc";
  };

  cargoNightly = funs.cargo {
    date = "2016-04-20";
    hash = "1harfb61rp1mwnh7mzr2x5y0xwvf5pd9r102k77pf2kmnsc7hq1r";
  };

  rustNightlyWithi686 = funs.rustcWithSysroots {
    rustc = rustcNightly;
    sysroots = [
      (funs.rust-std {
        date = "2016-04-20";
        hash = "08bcx86nkndmgi79vw12jj7s1a1gfzknl0g7i851jg81hq87y52k";
      })
      (funs.rust-std {
        date = "2016-04-20";
        hash = "0sl2h501wm8qa6qj124c4mqjxblh1l1hpdacpdkw02xsv2xxkq0s";
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
    cc32 gdb nasm qemu valgrind cargoNightly
    # We don't actually need prebuild std libs, but they're nice for testing libraries we
    # use, etc.
    (if release then rustcNightly else rustNightlyWithi686)
  ];

  src = ./.;

  enableParallelBuilding = true;
}
