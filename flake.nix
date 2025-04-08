{
  outputs = { self, nixpkgs }:
  let system = "x86_64-linux"; in
  let pkgs = import nixpkgs { inherit system; };
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs; [
        helix
        rustup
      ];
    };
  };
}
