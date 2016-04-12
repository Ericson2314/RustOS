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
    date = "2016-04-09";
    hash = "1dkks3g5r3v6j81hq73c8lmsngqyxjxjibpv799isd0lhxzs1jrr";
  };

  cargoNightly = funs.cargo {
    date = "2016-04-09";
    hash = "1y7wlplq1r88fijwg0831p0v1zny2fzmgmkjx8580jv36jh2kvbr";
  };

  rustNightlyWithi686 = funs.rustcWithSysroots {
    rustc = rustcNightly;
    sysroots = [
      (funs.rust-std {
        date = "2016-04-09";
        hash = "0xn8pqs1bakzh6apzrf8nas1yni4nsv90f0qpgb2cjvkjldksz4j";
      })
      (funs.rust-std {
        date = "2016-04-09";
        hash = "169gqmhmc71l17s2j6bhb8hpdbrnks8ckrzbimqm6x7gans728s8";
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
