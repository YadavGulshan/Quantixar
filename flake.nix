{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { 
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }: flake-utils.lib.eachSystem ["aarch64-darwin" "x86_64-linux"] (
    system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
            inherit system overlays;
      };

      rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      rustPlatform = pkgs.makeRustPlatform {
        cargo = rust;
        rustc = rust;
      };

      runtimeDependencies = with pkgs; [
        openssl
      ];

      frameworks = pkgs.darwin.apple_sdk.frameworks;

      buildDependencies = with pkgs; [
          libclang.lib
          clang
          pkg-config
          llvmPackages.bintools
          rustup
          rustPlatform.bindgenHook]
        ++ runtimeDependencies
        ++ lib.optionals stdenv.isDarwin [
          frameworks.Security
          frameworks.CoreServices
        ];

      developmentDependencies = with pkgs; [
          rust
        ]
        ++ buildDependencies;

      cargo-toml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
      with pkgs; {
        packages = flake-utils.lib.flattenTree rec {
          rocksdb-nix = rustPlatform.buildRustPackage rec {
            pname = cargo-toml.package.name;
            version = cargo-toml.package.version;
            
            env = { LIBCLANG_PATH = "${libclang.lib}/lib"; }
            // (lib.optionalAttrs (stdenv.cc.isClang && stdenv.isDarwin) { NIX_LDFLAGS = "-l${stdenv.cc.libcxx.cxxabi.libName}"; });
            
            src = ./.;
            cargoLock = {
                lockFile = builtins.toPath "./Cargo.lock";
            };

            nativeBuildInputs = buildDependencies;
            buildInputs = runtimeDependencies;

            doCheck = false;
          };
          default = rocksdb-nix;
        };
        devShells.default = mkShell {
          NIX_LDFLAGS = if stdenv.cc.libcxx != null && stdenv.cc.libcxx.cxxabi != null then "-l${stdenv.cc.libcxx.cxxabi.libName}" else "";
          HDF5_DIR = let
            hdf5 = pkgs.hdf5.overrideAttrs(old: {
              version = "1.10.7";

              src = fetchurl {
                url = "https://support.hdfgroup.org/ftp/HDF5/releases/hdf5-1.10/hdf5-1.10.7/src/hdf5-1.10.7.tar.bz2";
                sha256 = "sha256-AgGPrH5e/EltlTmjA8+0GSSl2t/6sF35gSCW4nPvpV4=";
              };

              postFixup = "";
            });
          in
            pkgs.symlinkJoin { name = "hdf5"; paths = [ hdf5 hdf5.dev ]; };

          buildInputs = developmentDependencies;
          shellHook = ''
            export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
            export RUSTFLAGS="-C link-args=-Wl,-rpath,/nix/store/dwsdfk42bdjrsxmhvairz58zzkk10v23-hdf5/lib"
         '';
        };
      }
  );
}
