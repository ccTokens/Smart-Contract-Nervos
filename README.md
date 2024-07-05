# CCToken Contracts

## Building

After tons of effort to unify the build process of contracts written in C and Rust, we finally make it works. Now, anyone
can simply execute `capsule build` to build all contracts.

### Config type ID of config-cell-type

It is crutial to config the type ID of config-cell-type when compiling for different evironments. The type ID is currently hard-coded in `contracts/c/structures.h` and switch by the the `contracts/c/Makefile`, so:

- If building for the mainnet, nothing else need to do.
- If building for the testnet, the `NET_TYPE ?= mainnet` in Makefile should be changed to `NET_TYPE ?= testnet`;
- If building for the unit tests, the line below `// dev env` in `contracts/c/structures.h` should be uncommented.

## Testing

> Due to the complexity of testing CKB contracts, the unit tests in this repository cover only type scripts. The dynamic libraries are not included in the tests.

If you want to run all the tests, you can use the following command:

```bash
capsule build --release

BINARY_VERSION=release capsule test
```

Run the tests against the release version of contracts can give you the maximum performance.

To run a specific test, use the following command:

```bash
capsule build

[RUST_LOG=true] [PRINT_TX=true] [PRINT_TEMPLATE=true] capsule test [test_name]
```

The `PRINT_TX` environment variable is used to print the transaction template. The `PRINT_TEMPLATE` environment variable is used to print the final transaction before verifying it in ckb-vm. The `test_name` is the name of any tests in the `tests` directory.

## Deploying

Since the capsule still lacks the capability to deploy multiple contracts repository, deployment requires using a binary tool called [CCToken Toolbox](https://github.com/dotbitHQ/ccToken-toolbox). For more information, please refer to that repository.
