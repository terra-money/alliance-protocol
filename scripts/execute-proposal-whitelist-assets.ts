import * as dotenv from 'dotenv'
import { MnemonicKey, LCDClient, MsgExecuteContract, ExecuteContractProposal, MsgSubmitProposal, Coins } from '@terra-money/feather.js';
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
    const lcd = new LCDClient({
        "pisco-1": {
            lcd: "https://pisco-lcd.terra.dev",
            chainID: "pisco-1",
            gasPrices: "0.15uluna",
            gasAdjustment: "1.2",
            prefix: process.env.ACC_PREFIX as string,
        }
    });

    const govAccountAddr = (await lcd.auth.moduleAccountInfo("pisco-1","gov"))?.baseAccount?.address;
    if (govAccountAddr == undefined) {
        console.log(`Something went wrong retreiving the governance account from on-chain`);
        return;
    }

    // Get all information from the deployer wallet
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC });
    const wallet = lcd.wallet(mk);
    const accAddress = wallet.key.accAddress(process.env.ACC_PREFIX as string);

    try {
        const hubAddress = fs.readFileSync('./scripts/.hub_address.log').toString('utf-8');

        const govProposal = new ExecuteContractProposal(
            "Whitelist an asset in Alliance Hub contract",
            "🕵🏼✌🏼👨🏼‍💻",
            govAccountAddr,
            hubAddress,
            {
                "whitelist_assets": {
                    "narwhal-1": [{
                        "native": "ibc/623CD0B9778AD974713317EA0438A0CCAA72AF0BBE7BEE002205BCA25F1CA3BA"
                    }]
                }
            },
        )

        const msgSubmitProposal = new MsgSubmitProposal(
            govProposal, 
            Coins.fromString("1000000000uluna"),
            accAddress
        );

        const tx = await wallet.createAndSignTx({
            msgs: [msgSubmitProposal],
            memo: "Whitelist an asset in Alliance Hub contract",
            chainID: process.env.CHAIN_ID as string,
        });
        const result = await lcd.tx.broadcastBlock(tx, process.env.CHAIN_ID as string);
        console.log(`Whitelist an asset in Alliance Hub contract submitted on chain
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