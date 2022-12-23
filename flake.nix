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

    linuxConfigs = pkgs.callPackage ./config.nix {};
    inherit (linuxConfigs) kernelArgs kernelConfig;

    kernelLib = kernelFlake.lib.builders {inherit pkgs;};

    configfile = kernelLib.buildKernelConfig {
      inherit
        (kernelConfig)
        kernelConfig
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
      inherit configfile nixpkgs;
    };

    linuxDev = pkgs.linuxPackagesFor kernelDrv;
    kernel = linuxDev.kernel;

    initramfs = kernelLib.buildInitramfs {
      inherit kernel;
      modules = [exmapModule];
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

    runQemu = kernelLib.buildQemuCmd {inherit kernel initramfs;};

    neovim = neovim-flake.lib.neovimConfiguration {
      inherit pkgs;
      modules = [
        {
          config = {
            vim.lsp = {
              enable = true;
              lightbulb.enable = true;
              lspSignature.enable = true;
              trouble.enable = true;
              nvimCodeActionMenu.enable = true;
              formatOnSave = true;
              rust = {
                enable = true;
                rustAnalyzerOpts = "";
              };
              nix.enable = true;
            };
            vim.statusline.lualine = {
              enable = true;
              theme = "onedark";
            };
            vim.visuals = {
              enable = true;
              nvimWebDevicons.enable = true;
              lspkind.enable = true;
              indentBlankline = {
                enable = true;
                fillChar = "";
                eolChar = "";
                showCurrContext = true;
              };
              cursorWordline = {
                enable = true;
                lineTimeout = 0;
              };
            };

            vim.theme = {
              enable = true;
              name = "onedark";
              style = "darker";
            };
            vim.autopairs.enable = true;
            vim.autocomplete = {
              enable = true;
              type = "nvim-cmp";
            };
            vim.filetree.nvimTreeLua.enable = true;
            vim.tabline.nvimBufferline.enable = true;
            vim.telescope = {
              enable = true;
            };
            vim.markdown = {
              enable = true;
              glow.enable = true;
            };
            vim.treesitter = {
              enable = true;
              autotagHtml = true;
              context.enable = true;
            };
            vim.keys = {
              enable = true;
              whichKey.enable = true;
            };
            vim.git = {
              enable = true;
              gitsigns.enable = true;
            };
          };
        }
      ];
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

      cargo
      cargo-edit
      cargo-audit
      cargo-tarpaulin
      clippy
      gdb

      runQemu
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
        nativeBuildInputs =
          nativeBuildInputs
          ++ [neovim.neovim];
      };
    };
}
