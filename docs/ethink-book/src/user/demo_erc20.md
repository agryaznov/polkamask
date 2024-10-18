# ERC20 Demo

### Prepare

Take just the same steps as described in the [flipper demo](demo.md#prepare)

### Set Up 

#### Build contract(s)

**ink! contract**

```bash 
cd dapp/contracts/erc20rlp.ink
cargo contract build 
```

#### Deploy contract

Make sure you've started our template node:

```bash
cargo run -- --dev
```

Now deploy the contract: 

```bash 
cd dapp/contracts/erc20rlp.ink
cargo contract instantiate -s 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133 --args=1_000_000_000_000 --config=Ecdsachain -x
```

(Notice we use *Alith's* private key here for transaction signing).

You should get the contract's code hash and address on the successful completion of the transaction: 

``` bash
Code hash 0x2a9248461edbc332459d7d7225ca1a607455d7b2aed2492e0ad070f6cc8d9ec4
Contract 0x3DeCA5F41730E50775eE7aFE5d6bdae8c82Ce54e
```

### 🚀 Run It! 
`TDB`

1. [ ] Add ERC20 to metamask (decimals=6?)
2. [ ] See balance displayed 
3. [ ] Transfer 
