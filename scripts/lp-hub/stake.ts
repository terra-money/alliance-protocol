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
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC });
    const wallet = lcd.wallet(mk);
    const accAddress = wallet.key.accAddress("terra");

    try {
        const hubAddress = fs.readFileSync('.lp-hub-addr.log').toString('utf-8');

        const stakeCoins = new MsgExecuteContract(
            accAddress,
            hubAddress,
            { "stake": {} },
            Coins.fromString("5000factory/terra1zdpgj8am5nqqvht927k3etljyl6a52kwqup0je/stDeck")
        );

        const stakeTokens = new MsgExecuteContract(
            accAddress,
            "terra16xkl47splqj964cxzm5q5g0aju09n53stauu22x4hfsgekam7z5qd26q70",
            {
                "send": {
                    "contract": hubAddress,
                    "amount": "5000",
                    "msg": ""
                }
            }
        );

        const stakeTokens2 = new MsgExecuteContract(
            accAddress,
            "terra1k2gv5ae4pk7ecc04qs9c5yqkw28cl09mqn85447amt5t2slm7uastaxagl",
            {
                "send": {
                    "contract": hubAddress,
                    "amount": "5000",
                    "msg": ""
                }
            }
        );
        const tx = await wallet.createAndSignTx({
            msgs: [stakeCoins, stakeTokens, stakeTokens2],
            memo: "Just Staking some tokens",
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