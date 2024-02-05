import * as dotenv from 'dotenv'
import { MnemonicKey, LCDClient, MsgSubmitProposal, Coins, MsgExecuteContract } from '@terra-money/feather.js';
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

        const govProposal = new MsgExecuteContract(
            govAccountAddr,
            hubAddress,
            {
                "modify_asset_pairs": [
                    {
                        asset_info: { native: "factory/terra1zdpgj8am5nqqvht927k3etljyl6a52kwqup0je/stDeck" },
                        reward_asset_info: { native: "uluna" },
                        delete: false
                    }, {
                        asset_info: { cw20: "terra15npavsvzqsnphnda67v5jpr2md4fp7gyexeffnv08wp8tlxn88xsjvxkgx" }, // VKR-ULUN-LP
                        reward_asset_info: { native: "uluna" },
                        delete: false
                    }, {
                        asset_info: { cw20: "terra15npavsvzqsnphnda67v5jpr2md4fp7gyexeffnv08wp8tlxn88xsjvxkgx" }, // VKR-ULUN-LP
                        reward_asset_info: { cw20: "terra167dsqkh2alurx997wmycw9ydkyu54gyswe3ygmrs4lwume3vmwks8ruqnv" },
                        delete: false
                    }, {
                        asset_info: { cw20: "terra1k2gv5ae4pk7ecc04qs9c5yqkw28cl09mqn85447amt5t2slm7uastaxagl" }, // WETH-ULUN-LP 
                        reward_asset_info: { native: "uluna" },
                        delete: false
                    }, {
                        asset_info: { cw20: "terra1k2gv5ae4pk7ecc04qs9c5yqkw28cl09mqn85447amt5t2slm7uastaxagl" }, // WETH-ULUN-LP 
                        reward_asset_info: { cw20: "terra167dsqkh2alurx997wmycw9ydkyu54gyswe3ygmrs4lwume3vmwks8ruqnv" },
                        delete: false
                    }
                ]
            }
        )

        const msgSubmitProposal = new MsgSubmitProposal(
            [govProposal as any],
            Coins.fromString("512000000uluna"),
            accAddress,
            "Just enabling some random assets in Alliance LP Hub Contract",
            "Just enabling some random assets in Alliance LP Hub Contract",
            "Just enabling some random assets in Alliance LP Hub Contract",
        );

        const tx = await wallet.createAndSignTx({
            msgs: [msgSubmitProposal],
            memo: "Just enabling some random assets in Alliance LP Hub Contract",
            chainID: "pisco-1",
        });
        const result = await lcd.tx.broadcastSync(tx, "pisco-1");
        console.log(`Proposal to whitelist some random assets in Alliance LP Hub Contract
        - Tx Hash: ${result.txhash}`);
    }
    catch (e) {
        console.log(e)
        return;
    }
}
init();