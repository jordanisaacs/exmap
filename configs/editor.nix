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
      clang.enable = true;
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
