import * as dotenv from 'dotenv'
import { MnemonicKey, LCDClient, MsgMigrateContract } from '@terra-money/feather.js';
import * as fs from 'fs';

dotenv.config()

const init = async () => {
    // Check if the hub contract is deployed
    // and with the proper information stored in the file
    if (!fs.existsSync('./scripts/.hub_address.log')
        || fs.readFileSync('./scripts/.hub_address.log').toString('utf-8') == "") {
        console.log(`Pleae deploy the hub contract first or add it's address to the ./scripts/.hub_address.log file to run this script`);
        return;
    }

    // Create the LCD Client to interact with the blockchain
    const lcd = LCDClient.fromDefaultConfig("testnet")

    // Get all information from the deployer wallet
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC});
    const wallet = lcd.wallet(mk);
    const accAddress = wallet.key.accAddress("terra");

    try {
        const hubAddress = fs.readFileSync('./scripts/.hub_address.log').toString('utf-8');
        const msgMigrateCode = new MsgMigrateContract(
            accAddress,
            hubAddress,
            9865,
            {}
        );

        const tx = await wallet.createAndSignTx({
            msgs: [msgMigrateCode],
            memo: "Migrate Alliance Hub",
            chainID: "pisco-1",
        });
        const result = await lcd.tx.broadcastBlock(tx, "pisco-1");
        console.log(`Migrate Alliance Hub submitted on chain
        - Tx Hash: ${result.txhash}`);
    }
    catch (e) {
        console.log(e)
        return;
    }
}

try {
    init();
}
catch (e) {
    console.log(e)
}2