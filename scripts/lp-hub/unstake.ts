import * as dotenv from 'dotenv'
import { MnemonicKey, LCDClient, MsgExecuteContract, Coins } from '@terra-money/feather.js';
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
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC });
    const wallet = lcd.wallet(mk);
    const accAddress = wallet.key.accAddress("terra");

    try {
        const hubAddress = fs.readFileSync('.lp-hub-addr.log').toString('utf-8');

        const unstakeCoins = new MsgExecuteContract(
            accAddress,
            hubAddress,
            {
                "unstake": {
                    "info": {
                        "cw20": "terra16xkl47splqj964cxzm5q5g0aju09n53stauu22x4hfsgekam7z5qd26q70"
                    },
                    "amount": "1000"
                }
            },
        );
        const unstakeTokens = new MsgExecuteContract(
            accAddress,
            hubAddress,
            {
                "unstake": {
                    "info": {
                        "cw20": "terra1k2gv5ae4pk7ecc04qs9c5yqkw28cl09mqn85447amt5t2slm7uastaxagl"
                    },
                    "amount": "1000"
                }
            },
        );
        const unstakeTokens2 = new MsgExecuteContract(
            accAddress,
            hubAddress,
            {
                "unstake": {
                    "info": {
                        "native": "factory/terra1zdpgj8am5nqqvht927k3etljyl6a52kwqup0je/stDeck"
                    },
                    "amount": "1000"
                }
            },
        );

        const tx = await wallet.createAndSignTx({
            msgs: [unstakeCoins, unstakeTokens, unstakeTokens2],
            memo: "Just unstaking some tokens",
            chainID: "pisco-1",
        });
        const result = await lcd.tx.broadcastSync(tx, "pisco-1");
        console.log(`Transaction executed successfully with hash:
        - Tx Hash: ${result.txhash}`);
    }
    catch (e) {
        console.log(e)
        return;
    }
}
init();