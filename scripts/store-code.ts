import * as dotenv from 'dotenv'
import { MnemonicKey, MsgStoreCode, LCDClient } from '@terra-money/feather.js';
import * as fs from 'fs';

dotenv.config()

const init = async () => {
    // Create the LCD Client to interact with the blockchain
    const lcd = LCDClient.fromDefaultConfig("testnet")

    // Get all information from the deployer wallet
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC });
    const wallet = lcd.wallet(mk);
    const accAddress = wallet.key.accAddress("terra");
    console.log(`Wallet address: ${accAddress}`)

    // Create the message and broadcast the transaction on chain
    let tx = await wallet.createAndSignTx({
        msgs: [new MsgStoreCode(
            accAddress,
            fs.readFileSync('./artifacts/alliance_hub.wasm').toString('base64')
        ),
        new MsgStoreCode(
            accAddress,
            fs.readFileSync('./artifacts/alliance_oracle.wasm').toString('base64')
        )
        ],
        memo: "Alliance Protocol Contracts",
        chainID: "pisco-1",
    });
    let result = await lcd.tx.broadcastSync(tx, "pisco-1");
    console.log(`Contracts stored on chain with the ID ${result.txhash}`);
}

try {
    init();
}
catch (e) {
    console.log(e)
}