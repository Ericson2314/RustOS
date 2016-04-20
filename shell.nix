{ release ? false }: (import <nixpkgs> {}).callPackage ./. { inherit release; }
