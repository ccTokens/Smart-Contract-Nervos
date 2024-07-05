#specify the target name
#NET_TYPE=testnet capsule build
#RUSTFLAGS="--cfg debug_assertions" capsule build -n config-cell-type --release

#for testnet
NET_TYPE=testnet RUSTFLAGS="--cfg debug_assertions" capsule build --release

#for mainnet
NET_TYPE=testnet RUSTFLAGS="--cfg debug_assertions" capsule build --release

