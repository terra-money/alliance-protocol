# Benchmark 
The smart contract has been deployed to [pisco-1 testnet](https://finder.terra.money/testnet/address/terra1uysfaxm4sjd7j35cw484w3ky3v6fkpffgrzv63mp6mj64xdamp2stf6hmt) and it has been benchmarked with the following data:

| Size   | Chains | Alliances | Data Size (kB) | Tx Cost (Luna) | Gas Used  |
|--------|--------|-----------|----------------|----------------|-----------|
| [XSmall](https://finder.terra.money/testnet/tx/4EFB6A2CA53C54449B303CF3C91593E161E12195E0C302E9504ECF67EBF00078)  | 8     | 16        | 5,5             | 0.406522       | 354,576   |
| [Small](https://finder.terra.money/testnet/tx/D04758D0E6B1DF1B910ACB0473A3C232273A12D06A670CA0DB4AF53CA9981ECB)  | 16     | 32        | 11             | 0.667318       | 574,760   |
| [Medium](https://finder.terra.money/testnet/tx/7BFCFDB5D378C58BFCF117660C57DC7C909D6EB45C316F86FFD4FD255EA8C5C7) | 32     | 64        | 21,9           | 1.188321       | 1,008,969 |
| [Large](https://finder.terra.money/testnet/tx/3DF693DAC85B3D0EAFFFCA580031DD81D106F01CE582DC7EAB5D3C14F41F833E)  | 64     | 124       | 43,7           | 2.230764       | 1,877,682 |
| [XLarge](https://finder.terra.money/testnet/tx/9BA0C5C18D6BC484112A7C15F5E1ECCBB4D80C1CF117895E08A375F182407325)  | 124     | 248       | 87,4           | 4.319313       | 3,615,282 |

Take in consideration that the XLarge dataset almost reach the limit of data we can submit on chain since the maximum gas for queries is 3 000 000 and the size of the data is quiet large. 

# Deployment Matix

|        | Network             | CodeID | Contract Address                                                 |
|--------|---------------------|--------|------------------------------------------------------------------|
| Oracle | testnet (pisco-1)   | 9408   | [terra1jf3nndysevley5p3wnajkjvjxcql9d00gpj4en3xwp7yrkrdqess48rr27](https://finder.terra.money/testnet/address/terra1jf3nndysevley5p3wnajkjvjxcql9d00gpj4en3xwp7yrkrdqess48rr27) |
| Hub    | testnet (pisco-1)   | 9573   | [terra1majrm6e6n0eg760n9fs4g5jvwzh4ytp8e2d99mfgzv2e7mjmdwxse0ty73](https://finder.terra.money/testnet/address/terra1majrm6e6n0eg760n9fs4g5jvwzh4ytp8e2d99mfgzv2e7mjmdwxse0ty73) |
| Oracle | mainnet (phoenix-1) | 1704   | [terra1nz8nl7wmwkzz94et2wy4xfrkglqwl7u3svp2mh6tdcwtzcdaysmq23fzpu](https://finder.terra.money/mainnet/address/terra1nz8nl7wmwkzz94et2wy4xfrkglqwl7u3svp2mh6tdcwtzcdaysmq23fzpu)                                                                 |
| Hub    | mainnet (phoenix-1) |        |                                                                  |

# Development

Considering the Rust is installed in your system you have to use the wasm32 compiler and install cargo-make. 

```sh
$ rustup default stable
$ rustup target add wasm32-unknown-unknown
$ cargo install --force cargo-make
```

There are few available commands to run on development:

Validate the code has been formatted correctly:
```sh
$ cargo make fmt
```

Run the tests written for the smart contracts
```sh
$ cargo make test
```

Lint the code 
```sh
$ cargo make lint
```

Generate json Schemas for each smart contract
```sh
$ cargo make schema
```

Build the code
```sh
$ cargo make build
```

Optimize the built code
```sh
$ cargo make optimize
```

# Deployment 

Before executing the following scripts, navigate to the scripts folder and execute `yarn` command to install the dependencies for the set of scripts. Also remember to set the following environment variables:

```sh
# Mnemonic of the account to deploy the contract with
MNEMONIC=
# Chain id where to deploy the contract
CHAIN_ID=
# Prefix of the acccounts where to deploy the smart contract 
ACC_PREFIX=
```

To deploy oracle and alliance hub smart contract:
```sh
$ cargo make deploy-oracle
$ cargo make deploy-hub
```
