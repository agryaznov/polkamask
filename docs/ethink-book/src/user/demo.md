# Demo 🧐

Our little demo consists of the three basic actions we complete on our *Substrate*-based 🦆-network using *MetaMask*:

1. **👛 Transfer Tokens**.

   With the MetaMask UI controls solely.

   This is the simplest action as we already have everything set up to do this. 
   Once launched the ethink! node with `cargo run -- --dev`, just open your MetaMask and make sure it is connected to our 🦆 network. You should see *Alith* account holding `10000000 🥚`. Go ahead and send some amount of eggs to *Goliath* or any other account you'd like to (set gas limit to `21000` as requested by MetaMask). 

2. **⚡ dApp (simple): Tokens Transfer**.

   Via *web3js*-based dApp used with MetaMask for signing transactions.
   
3. **🚀 dApp (advanced): ink! + MetaMask**.

   Call *ink!* smart contract via *web3js*-based dApp using  *MetaMask* for signing transactions.

For the actions 2,3 we have a simple [dApp](/dapp) which consists of a static [web page](https://github.com/agryaznov/ethink/blob/master/dapp/index.html) and of course our [**ink!** contract](https://github.com/agryaznov/ethink/blob/master/dapp/contracts/flipper.ink/lib.rs). This needs a bit of preparatory job to be done to set things up first.


### Prepare

Our 🦆-chain has _pallet-contracts_ on board and at the same time works with _Ethereum_ 20-bytes _Account_ format. The latter fact is required so that our node can understand *MetaMask*-signed transactions. But for the existing _ink!_ contracts tooling this is an unusual setting, as they're expected to work with 32-bytes long _Accounts_.  

For this reason, to work with our *ink!* contracts on this chain, we use a fork of _cargo-contract_ tool which speaks with our node the same language! Run this command to install it: 

``` bash
cargo install --git https://github.com/agryaznov/cargo-contract --branch ethink --force
```

### Set Up 

#### Build contract(s)

**ink! contract**

```bash 
cd dapp/contracts/flipper.ink
cargo contract build 
```

**Solidity contract (optional)**

> [!NOTE]
> In order to get Ethereum _web3js_ library work with our *ink!* contract, we **gotta make it believe it deals with an Ethereum contract**. 
> For that, we need to generate a metadata for our contract in the proper format. For the purposes of our PoC demo, we uploaded a ready-to use [JSON file](dapp/contracts/flipper.sol/build/contracts/) with that metadata. Though if you'd like you can install <a href="https://trufflesuite.com/" target="_blank">truffle</a> tool and build it yourself as described below.
>
> ❗**Keep in mind** that in the future this step is intended to be done by existing ink! tooling (e.g. _cargo-contract_).


``` bash
cd dapp/contracts/flipper.sol
truffle build
```

#### Deploy contract

Make sure you've started our template node:

```bash
cargo run -- --dev
```

Then deploy the contract: 

```bash 
cd dapp/contracts/flipper.ink
cargo contract instantiate -s 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133 --args=false -x
```

(Notice we use *Alith's* private key here for transaction signing).

You should get the contract's code hash and address on the successful completion of the transaction: 

``` bash
 Code hash 0x417370a73c71e0787a6da2c8b34ee035517175ed28beb1328461b642670975b7
 Contract 0xAc7dA28B0A6e94dEc4c9D2bFA6917Ff476e6a944
```

#### Prepare dApp

``` bash
cd dapp/client
npm i
npm start
```

For the demo purposes we made our dApp dead simple. You might need to put actual deployed contract address here to its source code (needed only of you changed the contract source): 

<details>
https://github.com/agryaznov/ethink/blob/f4e2624c0cfce0d77fb9eb980cb9ad44671ee1d4/dapp/client/src/index.js#L20-L24
</details>


### 🚀 Run It! 

Once you have your chain node and dApp started, open your browser at [http://localhost:8080/client/](http://localhost:8080/client/) to load our dApp: 

![dApp home page](/images/dapp-0.png)

You should see the MetaMask pop-up asking for permissions. Allow it to use _Alith_ account on this site:

![MetaMask 1](/images/dapp-2.png)


Click on MetaMask icon and make sure it's connected to our Duck chain. You should see *Alith's* balance in eggs:

![MetaMask 1](/images/dapp-1.png)


#### Making 2. **⚡ dApp (simple): tokens transfer**:  

Now go ahead and click on `Send Tokens` button! 

![MetaMask 1](/images/dapp-3.png)

Confirm the transaction... and, in a moment later you should see one egg has been sent to Goliath!

![MetaMask 1](/images/dapp-4.png)

Cool, we have just used an web3js dApp to send some eggs between accounts on our Duck network!  
Keep going, in a few moments we'll call our ink! contract with it!

#### Making 3. **🚀 dApp (advanced): ink! + MetaMask**.  


First, check the current state of our contract with this command:

``` bash
cargo contract call -s 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133 --contract 0xAc7dA28B0A6e94dEc4c9D2bFA6917Ff476e6a944 --message get
```

We see that current Flipper's state is `false`:

``` bash
 Result Success!
 Reverted false
 Data Ok(false)
```

<details>
<summary>
We can also do the same check via PolkadotJS Apps as usual: 
</summary>

<mark>TBD: add screenshots</mark>

</details>


And finally, the moment of truth has come. 🥁 Can we **really** call our **ink!** contract from **MetaMask**? Well, let's see. 

Open our [dApp page](http://localhost:8080/client/) again. Click on the `Call Contract` button. 

![MetaMask 1](/images/dapp-5.png)

Click on `Confirm` and wait until your transaction made it to the block: 

![MetaMask 1](/images/dapp-6.png)

Now check the state again with *cargo-contract*, and... 

``` bash
 Result Success!
 Reverted false
 Data Ok(true)
```

🎉 **Congratulations, you have just called your *ink!* contract via *MetaMask*, and it just worked!**

> [!TIP]
> 💡 Looks simple, right? 
> 
> 🧠 Under the hood though, that was an amusing journey your transaction had made through your node's exposed custom Ethereum RPC, then it got transformed through your network's Runtime RPC, got into the transaction pool as a pallet-ethink extrinsic, then got into the block and processes by that pallet which understood it's a transaction for pallet-contracts, transformed it again to the corresponding dispatchable which finally made it to your contract!
>
> Whoa, what a long way isn't it? But let's put the details off for now and just enjoy the moment!  
> The design technicalities are to be explained in the ethink! docs, stay tuned!
