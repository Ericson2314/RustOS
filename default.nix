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
    date = "2016-06-09";
    hash = "1hnkw7gd7nihc3jqkckp9hgashdc8vmvc92qvxfyalxrp84fhyr1";
  };

  cargoNightly = funs.cargo {
    date = "2016-06-09";
    hash = "1p0xkpfk66jq0iladqfrhqk1zc1jr9n2v2lqyf7jjbrmqx2ja65i";
  };

  stdDate = "2016-06-09";

  rustNightlyWithi686 = funs.rustcWithSysroots {
    rustc = rustcNightly;
    sysroots = [
      (funs.rust-std {
        date = stdDate;
        hash = "1261y0wqczipn0fv3q3d1yl6q3djisrlji4fs94sabzpwwsjq4cc";
      })
      (funs.rust-std {
        date = stdDate;
        hash = "1z0q7qxvf6rc2xa4bmjfa51ya2yq7ba2wvcgz5rvyv36cjdnsnrk";
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
  ] ++ stdenv.lib.optionals (!release) [
    rustcNightly.doc
  ];

  src = ./.;

  enableParallelBuilding = true;
}
