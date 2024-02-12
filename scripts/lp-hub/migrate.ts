import * as dotenv from 'dotenv'
import { MnemonicKey, LCDClient, MsgMigrateContract } from '@terra-money/feather.js';
import * as fs from 'fs';

dotenv.config()

const init = async () => {
    // Check if the hub contract is deployed
    // and with the proper information stored in the file
    if (!fs.existsSync('.lp-hub-addr.log')
        || fs.readFileSync('.lp-hub-addr.log').toString('utf-8') == "") {
        console.log(`Pleae deploy the hub contract first or add it's address to the .lp-hub-addr.log file to run this script`);
        return;
    }

    // Create the LCD Client to interact with the blockchain
    const lcd = new LCDClient({
        "pisco-1": {
            chainID : "pisco-1",
            gasAdjustment : 1.5,
            gasPrices : {
                uluna: 0.02
            },
            lcd: "http://192.168.2.101:1317/",
            prefix: "terra"
        }
    });

    // Get all information from the deployer wallet
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC});
    const wallet = lcd.wallet(mk);
    const accAddress = wallet.key.accAddress("terra");

    try {
        const hubAddress = fs.readFileSync('.lp-hub-addr.log').toString('utf-8');
        const hubCode = fs.readFileSync('.lp-hub-code-id.log').toString('utf-8');

        const tx = await wallet.createAndSignTx({
            msgs: [new MsgMigrateContract(
                accAddress,
                hubAddress,
                Number(hubCode),
                {}
            )],
            memo: "Migrate Alliance LP Hub",
            chainID: "pisco-1",
        });
        const result = await lcd.tx.broadcastSync(tx, "pisco-1");
        console.log(`Migration for Alliance LP Hub submitted on chain ${result.txhash}`);
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