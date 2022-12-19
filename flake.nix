{
  description = "A very basic flake";

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    kernel = pkgs.linuxPackages_latest.kernel;

    exmap = pkgs.stdenv.mkDerivation {
      name = "exmap";
      src = ./module;

      buildInputs = [pkgs.nukeReferences];
      kernel = kernel.dev;
      kernelVersion = kernel.modDirVersion;

      buildPhase = ''
        make -s "KDIR=$kernel/lib/modules/$kernelVersion/build" modules
      '';

      installPhase = ''
        mkdir -p $out/lib/modules/$kernelVersion/misc
        for x in $(find . -name '*.ko'); do
          nuke-refs $x
          cp $x $out/lib/modules/$kernelVersion/misc/
        done
      '';
    };
  in {
    packages.x86_64-linux = {
      inherit exmap;
      default = exmap;
    };
  };
}
