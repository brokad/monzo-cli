{
  description = "A CLI for your Monzo accounts";

  inputs = {
    naersk.url = "github:nmattia/naersk";
    mozillapkgs = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, naersk, mozillapkgs }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;

      # Get a specific rust version
      mozilla = pkgs.callPackage (mozillapkgs + "/package-set.nix") {};
      rust = (mozilla.rustChannelOf {
        date = "2021-09-21";
        channel = "nightly";
        sha256 = "0wq76fdskrjxmp6hcl419lz4bilr22qmpp6n88xbi41yf12himk9";
      }).rust;

      naersk-lib = naersk.lib."x86_64-linux".override {
        cargo = rust;
        rustc = rust;
      };
    in {
      packages.x86_64-linux.monzo-cli = naersk-lib.buildPackage {
        pname = "monzo-cli";
        buildInputs = with pkgs; [
          openssl.dev
          pkgconfig
        ];
        root = ./.;
      };

      defaultPackage.x86_64-linux = self.packages.x86_64-linux.monzo-cli;

      #devShell = pkgs.mkShell {
      #  nativeBuildInputs = [ rust ];
      #};
  };
}
