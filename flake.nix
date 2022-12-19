{
  inputs = {
    nixpkgs.url = "github:jordanisaacs/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    exmap = {
      url = "github:jordanisaacs/exmap-module";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    neovim-flake.url = "github:jordanisaacs/neovim-flake";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    neovim-flake,
    exmap,
    crate2nix,
    ...
  }: let
    system = "x86_64-linux";
    overlays = [
      rust-overlay.overlays.default
      (self: super: let
        rust = super.rust-bin.stable.latest.default;
      in {
        rustc = rust;
        cargo = rust;
      })
    ];
    pkgs = import nixpkgs {
      inherit system overlays;
    };

    exmapMod = exmap.packages.${system}.exmap;

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

    # inherit
    #   (import "${crate2nix}/tools.nix" {inherit pkgs;})
    #   generatedCargoNix
    #   ;
    # pkg =
    #   (
    #     import
    #     (generatedCargoNix {
    #       inherit name;
    #       src = ./.;
    #     })
    #     {inherit pkgs;}
    #   )
    #   .workspaceMembers
    #   .client
    #   .build;

    nativeBuildInputs = with pkgs; [
      rustc
      rust-bindgen
      rustPlatform.bindgenHook

      cargo
      cargo-edit
      cargo-audit
      cargo-tarpaulin
      clippy

      liburing
      exmapMod
    ];
    #buildInputs = with pkgs; [clang llvmPackages.libclang.lib stdenv.cc.libc];
  in
    with pkgs; {
      packages.${system} = {
        ${name} = pkg;
        default = pkg;
      };

      devShells.${system}.default = mkShell {
        NIX_CFLAGS_COMPILE = "-I${pkgs.linuxPackages_latest.kernel.dev}/lib/modules/${pkgs.linuxPackages_latest.kernel.modDirVersion}/source/include";
        nativeBuildInputs =
          nativeBuildInputs
          ++ [neovim.neovim];
      };
    };
}
