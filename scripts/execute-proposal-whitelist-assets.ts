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
    const lcd = LCDClient.fromDefaultConfig("mainnet");

    const govAccountAddr = (await lcd.auth.moduleAccountInfo("phoenix-1","gov"))?.baseAccount?.address;
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
            "Whitelist assets in Alliance Hub",
            "This proposal will whitelist White Whale ampWHALE and boneWHALE ibc asset in the Alliance Hub contract",
            govAccountAddr,
            hubAddress,
            {
                "whitelist_assets": {
                    "narwhal-1": [{
                        "native": "ibc/B3F639855EE7478750CC8F82072307ED6E131A8EFF20345E1D136B50C4E5EC36"
                    },{
                        "native": "ibc/517E13F14A1245D4DE8CF467ADD4DA0058974CDCC880FA6AE536DBCA1D16D84E"
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