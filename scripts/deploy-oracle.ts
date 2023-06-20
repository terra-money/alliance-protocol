import * as dotenv from 'dotenv'
import { MnemonicKey, MsgStoreCode, MsgInstantiateContract, LCDClient, Coins } from '@terra-money/feather.js';
import * as fs from 'fs';

dotenv.config()

const init = async () => {
    // Variable will populate after storing the code on chain
    let codeId: number;

    // Create the LCD Client to interact with the blockchain
    const lcd = new LCDClient({
        "test-1": {
            lcd: "http://localhost:1316",
            chainID: "test-1",
            gasPrices: "0.15uluna",
            gasAdjustment: "1.2",
            prefix: process.env.ACC_PREFIX as string,
        }
    });
    // Get all information from the deployer wallet
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC });
    const wallet = lcd.wallet(mk);
    const accAddress = wallet.key.accAddress(process.env.ACC_PREFIX as string);
    console.log(`Wallet address: ${accAddress}`)

    // Create the message and broadcast the transaction on chain
    try {
        const msgStoreCode = new MsgStoreCode(
            accAddress,
            fs.readFileSync('./artifacts/alliance_oracle.wasm').toString('base64')
        );
        let tx = await wallet.createAndSignTx({
            msgs: [msgStoreCode],
            memo: "Alliance Oracle Contract",
            chainID: process.env.CHAIN_ID as string,
        });

        let result = await lcd.tx.broadcastBlock(tx, process.env.CHAIN_ID as string);
        codeId = Number(result.logs[0].events[1].attributes[1].value);
        console.log(`Smart contract deployed with 
        - Code ID: ${codeId}
        - Tx Hash: ${result.txhash}`);
        await new Promise(resolve => setTimeout(resolve, 1000));
    }
    catch (e) {
        console.log(e);
        return;
    }

    try {
        // Instantiate the transaction and broadcast it on chain
        const msgInstantiateContract = new MsgInstantiateContract(
            accAddress,
            accAddress,
            codeId,
            {
                "controller_addr": accAddress,
                "governance_addr": "terra10d07y265gmmuvt4z0w9aw880jnsr700juxf95n",
                "data_expiry_seconds": 600,
            },
            new Coins(),
            "Create an oracle contract"
        );

        const tx = await wallet.createAndSignTx({
            msgs: [msgInstantiateContract],
            memo: "Create an Alliance Oracle Contract",
            chainID: process.env.CHAIN_ID as string,
        });
        const result = await lcd.tx.broadcastBlock(tx, process.env.CHAIN_ID as string);
        const contractAddress = result.logs[0].events[0].attributes[0].value;
        console.log(`Smart contract instantiated with 
        - Code ID: ${codeId}
        - Tx Hash: ${result.txhash}
        - Contract Address: ${contractAddress}`);

        fs.writeFileSync('./scripts/.oracle_address.log', contractAddress);
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