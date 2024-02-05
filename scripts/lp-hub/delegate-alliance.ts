import * as dotenv from 'dotenv'
import { MnemonicKey, LCDClient, MsgExecuteContract } from '@terra-money/feather.js';
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
    const lcd = LCDClient.fromDefaultConfig("testnet");

    // Get all information from the deployer wallet
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC});
    const wallet = lcd.wallet(mk);
    const accAddress = wallet.key.accAddress("terra");

    try {
        const hubAddress = fs.readFileSync('.lp-hub-addr.log').toString('utf-8');
        const msgExecute = new MsgExecuteContract(
            accAddress,
            hubAddress, 
            {
                "alliance_delegate" : {
                    "delegations": [{
                        "validator": "terravaloper1zdpgj8am5nqqvht927k3etljyl6a52kwqndjz2",
                        "amount": "10000000"
                    }]
                }
            },
        );

        const tx = await wallet.createAndSignTx({
            msgs: [msgExecute],
            memo: "Stake to Alliance Module",
            chainID: "pisco-1",
        });
        const result = await lcd.tx.broadcastSync(tx, "pisco-1");
        console.log(`Stake to Alliance Module submitted on chain
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
}