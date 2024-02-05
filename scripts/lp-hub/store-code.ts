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

    try {

        // Create the message and broadcast the transaction on chain
        let tx = await wallet.createAndSignTx({
            msgs: [new MsgStoreCode(
                accAddress,
                fs.readFileSync('./artifacts/alliance_lp_hub.wasm').toString('base64')
            )],
            memo: "Alliance LP Hub Contracts",
            chainID: "pisco-1",
        });
        let result = await lcd.tx.broadcastSync(tx, "pisco-1")
        await new Promise(resolve => setTimeout(resolve, 7000))
        const txResult : any = await lcd.tx.txInfo(result.txhash, "pisco-1");
        const codeId = txResult.logs[0].events[1].attributes[1].value;
        console.log("Contract stored on chain with CodeId", codeId)
        fs.writeFileSync(".lp-hub-code-id.log", codeId);
    }
    catch (e) {
        console.log(e)
    }
};
init();