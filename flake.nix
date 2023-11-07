{
  description = "service spo provisionierung";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        rust-stable = pkgs.rust-bin.stable.latest.default;

        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
      in
      with pkgs; {
        devShell =
          pkgs.mkShell {
            buildInputs = [
              djlint
              nil
              nixpkgs-fmt
              nodejs_20
              nodePackages.prettier
              rust-analyzer
              rust-stable
              vscode-extensions.vadimcn.vscode-lldb
            ];
            shellHook = ''
              export PATH=~/.cargo/bin:$PATH
              export PATH=${vscode-extensions.vadimcn.vscode-lldb}/share/vscode/extensions/vadimcn.vscode-lldb/adapter:$PATH
            '';
          };
      });
}
