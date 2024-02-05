import * as dotenv from 'dotenv'
import { MnemonicKey, LCDClient, MsgInstantiateContract, Coins } from '@terra-money/feather.js';
import fs from "fs";

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
        const tx = await wallet.createAndSignTx({
            msgs: [new MsgInstantiateContract(
                accAddress,
                accAddress,
                12893,
                {
                    governance: "terra10d07y265gmmuvt4z0w9aw880jnsr700juxf95n",
                    controller: accAddress,
                    astro_reward_denom: "terra167dsqkh2alurx997wmycw9ydkyu54gyswe3ygmrs4lwume3vmwks8ruqnv",
                    astro_incentives_addr: "terra1ujqta8jx4w7z224q0heunfx4rz57e92kkeyrgmry3yz2qf5z3xlsnrk0eq",
                    alliance_reward_denom: "uluna",
                },
                Coins.fromString("10000000uluna"),
                "Instantiate Alliance LP Hub Contract",
            )],
            memo: "Alliance LP Hub",
            chainID: "pisco-1",
        });
        const result = await lcd.tx.broadcastSync(tx, "pisco-1")
        console.log(`Instantiate tx hash ${result.txhash}`);
        await new Promise(resolve => setTimeout(resolve, 7000))
        const txResult : any = await lcd.tx.txInfo(result.txhash, "pisco-1");
        const contractAddr = txResult.logs[0].eventsByType?.instantiate._contract_address[0];
        console.log("Contract stored on chain with contractAddr", contractAddr)
        fs.writeFileSync(".lp-hub-addr.log", contractAddr);
    }
    catch (e) {
        console.log(e)
    }
}
init();