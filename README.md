## Development

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

Build the code
```sh
$ cargo make build
```

Optimize the built code
```sh
$ cargo make optimize
```

## Deployment 

Before executing the following scripts, navigate to the scripts folder and execute `yarn` command to install the dependencies for the set of scripts. Also remember to set the following environment variables:

```sh
# Mnemonic of the account to deploy the contract with
MNEMONIC=
# Chain id where to deploy the contract
CHAIN_ID=
# Prefix of the acccounts where to deploy the smart contract 
ACC_PREFIX=
```

To deploy the oracle smart contract it can be done by executing the following command:
```sh
$ cargo make deploy-oracle
```
