import * as dotenv from 'dotenv'
import { MnemonicKey, MsgStoreCode, MsgInstantiateContract, LCDClient, Coins } from '@terra-money/feather.js';
import * as fs from 'fs';

dotenv.config()

const init = async () => {
    // Variable will populate after storing the code on chain
    let codeId: number;

    // Create the LCD Client to interact with the blockchain
    const lcd = LCDClient.fromDefaultConfig("testnet")

    // Get all information from the deployer wallet
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC });
    const wallet = lcd.wallet(mk);
    const accAddress = wallet.key.accAddress("terra");
    console.log(`Wallet address: ${accAddress}`)

    // Create the message and broadcast the transaction on chain
    const msgStoreCode = new MsgStoreCode(
        accAddress,
        fs.readFileSync('./artifacts/alliance_oracle.wasm').toString('base64')
    );
    let tx = await wallet.createAndSignTx({
        msgs: [msgStoreCode],
        memo: "Alliance Protocol Oracle",
        chainID: "pisco-1",
    });

    let result = await lcd.tx.broadcastBlock(tx, "pisco-1");
    codeId = Number(result.logs[0].events[1].attributes[1].value);
    console.log(`Smart contract deployed with 
    - Code ID: ${codeId}
    - Tx Hash: ${result.txhash}`);

    await new Promise(resolve => setTimeout(resolve, 3000));

    // Instantiate the transaction and broadcast it on chain
    const msgInstantiateContract = new MsgInstantiateContract(
        accAddress,
        accAddress,
        codeId,
        {
            "controller_addr": accAddress,
            "data_expiry_seconds": 86400,   // 24h
        },
        new Coins(),
        "Create Alliance Protocol Oracle"
    );

    tx = await wallet.createAndSignTx({
        msgs: [msgInstantiateContract],
        memo: "Create an Alliance Oracle Contract",
        chainID: "pisco-1",
    });
    result = await lcd.tx.broadcastBlock(tx, "pisco-1");
    let contractAddress = result.logs[0].events[0].attributes[0].value;
    console.log(`Alliance Oracle smart contract instantiated with 
    - Code ID: ${codeId}
    - Tx Hash: ${result.txhash}
    - Contract Address: ${contractAddress}`);

    fs.writeFileSync('./scripts/.oracle_address.log', contractAddress);
}

try {
    init();
}
catch (e) {
    console.log(e)
}