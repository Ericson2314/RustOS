{ stdenv, gdb, nasm, qemu, rustcNightly }:

stdenv.mkDerivation {
  name = "RustOS";

  nativeBuildInputs = [ gdb nasm qemu rustcNightly.rustc rustcNightly.cargo ];
  src = ./.;

  enableParallelBuilding = true;
}
