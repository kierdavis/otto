{
  inputs = {
    nixpkgs = {
      type = "github";
      owner = "NixOS";
      repo = "nixpkgs";
      ref = "release-25.11";
    };
  };

  outputs = inputs @ { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          alsa-lib
          (alsa-utils.override { withPipewireLib = false; })
          cargo
          pkg-config
          rustfmt
        ];
        shellHook = ''export PS1="\n\[\033[1;32m\][otto]\[\033[0m\] ''${PS1#\\n}"'';
      };
    };
}
