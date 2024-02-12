import * as dotenv from 'dotenv'
import { MnemonicKey, LCDClient, MsgSubmitProposal, Coins, MsgExecuteContract, MsgCreateAlliance } from '@terra-money/feather.js';
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

    const govAccountAddr = (await lcd.auth.moduleAccountInfo("pisco-1", "gov"))?.baseAccount?.address;
    if (govAccountAddr == undefined) {
        console.log(`Something went wrong retreiving the governance account from on-chain`);
        return;
    }

    // Get all information from the deployer wallet
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC });
    const wallet = lcd.wallet(mk);
    const accAddress = wallet.key.accAddress("terra");

    try {
        const hubAddress = fs.readFileSync('.lp-hub-addr.log').toString('utf-8');

        const govProposal = new MsgCreateAlliance(
            govAccountAddr,
            "factory/" + hubAddress + "/AllianceLP",
            "10000000000000000",
            "0",
            "10000000000000000",
            undefined,
            {
                max: "10000000000000000",
                min: "10000000000000000",
            }
        )

        const msgSubmitProposal = new MsgSubmitProposal(
            [govProposal as any],
            Coins.fromString("512000000uluna"),
            accAddress,
            "Just whitelisting an alliance",
            "Just whitelisting an alliance",
            "Just whitelisting an alliance",
        );

        const tx = await wallet.createAndSignTx({
            msgs: [msgSubmitProposal],
            memo: "Just whitelisting an alliance",
            chainID: "pisco-1",
        });
        const result = await lcd.tx.broadcastSync(tx, "pisco-1");
        console.log(`Proposal to whitelist an Alliance
        - Tx Hash: ${result.txhash}`);
    }
    catch (e) {
        console.log(e)
        return;
    }
}
init();