{
  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    neovim-flake.url = "github:jordanisaacs/neovim-flake";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
    kernelFlake = {
      url = "github:jordanisaacs/kernel-module-flake";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    neovim-flake,
    crate2nix,
    kernelFlake,
    ...
  }: let
    system = "x86_64-linux";
    overlays = [
      rust-overlay.overlays.default
      (self: super: let
        rust = super.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {extensions = ["rust-src" "miri"];});
      in {
        rustc = rust;
        cargo = rust;
      })
    ];
    pkgs = import nixpkgs {
      inherit system overlays;
    };

    enableGdb = true;

    linuxConfigs = pkgs.callPackage ./configs/kernel.nix {};
    inherit (linuxConfigs) kernelArgs kernelConfig;

    kernelLib = kernelFlake.lib.builders {inherit pkgs;};

    configfile = kernelLib.buildKernelConfig {
      inherit
        (kernelConfig)
        generateConfigFlags
        structuredExtraConfig
        ;
      inherit kernel nixpkgs;
    };

    kernelDrv = kernelLib.buildKernel {
      inherit
        (kernelArgs)
        src
        modDirVersion
        version
        ;

      inherit configfile nixpkgs enableGdb;
    };

    linuxDev = pkgs.linuxPackagesFor kernelDrv;
    kernel = linuxDev.kernel;

    modules = [exmapModule];
    initramfs = kernelLib.buildInitramfs {
      inherit kernel modules;
      extraBin = {
        exmap = "${exmapExample}/bin/exmap";
      };
      extraInit = ''
        insmod modules/exmap.ko
        mknod -m 666 /dev/exmap c 254 0
      '';
    };

    buildExmapModule = kernel:
      (kernelLib.buildCModule {inherit kernel;} {
        name = "exmap";
        src = ./module;
        dontStrip = true;
      })
      .overrideAttrs (old: {
        outputs = ["out" "dev"];
        installPhase =
          old.installPhase
          + ''
            mkdir -p $dev/include
            cp -r linux $dev/include
          '';
      });

    exmapModule = buildExmapModule kernel;

    runQemu = kernelLib.buildQemuCmd {inherit kernel initramfs enableGdb;};
    runGdb = kernelLib.buildGdbCmd {inherit kernel modules;};

    neovim = neovim-flake.lib.neovimConfiguration {
      inherit pkgs;
      modules = [./configs/editor.nix];
    };

    exmapExample = let
      buildRustCrateForPkgs = pkgs:
        pkgs.buildRustCrate.override {
          defaultCrateOverrides =
            pkgs.defaultCrateOverrides
            // {
              exmap = attrs: {
                NIX_CFLAGS_COMPILE = compileFlags;
                buildInputs = [pkgs.rustPlatform.bindgenHook exmapModule.dev];
              };
            };
        };
      generatedBuild = pkgs.callPackage ./Cargo.nix {
        inherit buildRustCrateForPkgs;
      };
    in
      generatedBuild
      .rootCrate
      .build;

    nativeBuildInputs = with pkgs; [
      rustc
      rust-bindgen
      rustPlatform.bindgenHook

      pkgs.crate2nix
      exmapModule

      cargo
      cargo-edit
      cargo-audit
      cargo-tarpaulin
      clippy

      bear

      runQemu
      runGdb
    ];

    compileFlags = "-I${kernel.dev}/lib/modules/${kernel.modDirVersion}/source/include";
  in
    with pkgs; {
      lib = {
        inherit buildExmapModule;
      };

      packages.${system} = {
        inherit exmapExample exmapModule;
      };

      devShells.${system}.default = mkShell {
        NIX_CFLAGS_COMPILE = compileFlags;
        KERNEL = kernel.dev;
        KERNEL_VERSION = kernel.modDirVersion;
        nativeBuildInputs =
          nativeBuildInputs
          ++ [neovim.neovim];
      };
    };
}
