# Quickstart

Fetch the project sources: 

```bash
git clone https://github.com/agryaznov/ethink.git
cd ethink
```

Run tests:

```bash
cargo test
```

Inject a well-known keypair of [Baltathar](/developer/known-accounts.html) into ethink! node's keystore:  
<mark>*(This step is needed only of you want to sign transactions on the node side)*</mark>  

```bash
cargo run -- key insert --dev --key-type "ethi" -d tmp --scheme ecdsa
```

Start the ethink! development node: 

```bash
cargo run -- --dev
```

Open your MetaMask and add a new network:

+ **Network name**: Duck 🦆 
+ **New RPC URL**: http://localhost:9944
+ **Chain ID**: 42
+ **Currency symbol**: 🥚

Import a couple of pre-funded *well-known* [development accounts](/developer/known-accounts.md) into your MetaMask in order to be able to sign and send transactions. 

> [!CAUTION]
> It is **highly recommended** to use a separate MetaMask instance for this (e.g. in a dedicated <a href="https://support.mozilla.org/en-US/kb/profiles-where-firefox-stores-user-data" target="_blank">browser profile</a>), not to mix the development accounts (whose private keys **are compromised** by design) with your real money-holding accounts. 

**That's it!** You should right away be able to communicate with the Duck 🦆 chain using your MetaMask. Let's see it in action, as described in [next](user/demo.md) section. 

