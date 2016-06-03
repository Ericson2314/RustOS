{ stdenv, callPackage, fetchFromGitHub, runCommand, makeWrapper, pkgsi686Linux
, gdb, nasm, qemu, valgrind
, release ? true
}:

let
  settings = fetchFromGitHub {
    owner = "Ericson2314";
    repo = "nixos-configuration";
    rev = "3b71fecbd51b28f58511f09deb0ed751e2c03d8d";
    sha256 = "1g8gdfscimjmwgqdzrj56f24ki7zn7a7nh8jq3v0a565vifq7wrw";
  };

  funs = callPackage "${settings}/user/.nixpkgs/rust-nightly.nix" { };

  rustcNightly = funs.rustc {
    date = "2016-05-07";
    hash = "0vpzwysgy2x94d8s2m87q9krldcpkwmqdma67ig93yhnw5z7iggf";
  };

  cargoNightly = funs.cargo {
    date = "2016-05-06";
    hash = "0irmd46i62jvhk6cprg6mq3bf5b8qsxc07vqhcgfnzzra9nz84gg";
  };

  stdDate = "2016-05-07";

  rustNightlyWithi686 = funs.rustcWithSysroots {
    rustc = rustcNightly;
    sysroots = [
      (funs.rust-std {
        date = stdDate;
        hash = "1gsdzvym1piy6ak9hz39hzjmaa6fg11p458dqb2anj2br5x76mq2";
      })
      (funs.rust-std {
        date = stdDate;
        hash = "1wa0bif9s48hkc77b13lam42xhgghf8gjrvl0mf5wx8p1dyxgiqw";
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
  ] ++ stdenv.lib.optionalAttrs (!release) [
    rustcNightly.doc
  ];

  src = ./.;

  enableParallelBuilding = true;
}
