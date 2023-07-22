<div align="center">

  <h1><code>Chain DB</code></h1>

  <p>
    <strong>Story-driven Database</strong>
  </p>
  
  <h3>
    <!-- <a href="https://borsh.io">Website</a> -->
    <a href="https://github.com/wpdas/chain-db">About</a>
    <span> | </span>
    <a href="https://github.com/wpdas/chain-db#download">Download</a>
    <span> | </span>
    <a href="https://github.com/wpdas/chain-db#libraries-client">Libraries</a>
  </h3>
</div>

Chain DB is a Story-driven database. This new type of system uses some features used in blockchain technology. Each change generates a transaction that is saved in a block. The network works centrally, so persistent data is not decentralized.

The way data is read resembles the format of blockchains as well. The generated blocks are immutable. New data or updated data are saved in newer blocks, in this way, it is possible to recover all data from a table throughout its life.

This database has some features by default, such as: create user account, get user account, transfer units and get transfer records as well as the main feature that is tables.

The `unit` property present in each user's account can be anything the project wants, it can be a type of currency, item, resource.

We are just at the beginning of the implementation of this database. Let me know if you want to contribute to this project, contact me here or via my email: wendersonpdas@gmail.com.

## Download

Go to [Releases](https://github.com/wpdas/chain-db/releases) page and download the binary according to the desired archtecure.

| Binary                                                                                                     | OS        | Devices                              |
| ---------------------------------------------------------------------------------------------------------- | --------- | ------------------------------------ |
| [chain-db-osx](https://github.com/wpdas/chain-db/releases/download/v0.0.2-alpha/chain-db-osx.chain-db-osx) | macOS 11+ | Apple                                |
| [chain-db-arm64](https://github.com/wpdas/chain-db/releases/download/v0.0.2-alpha/chain-db-arm64)          | Linux     | Raspberry Pi3, Raspberry Pi4, Others |

## Libraries (client)

Below are links to libraries (client) to be used with your preferred programming language.

Although some libraries work on the client side, it is recommended to use it on the server side since it is a database transiting sensitive data.

| Platform               | Repository                                          |
| ---------------------- | --------------------------------------------------- |
| Rust                   | [chain-db-rs](https://github.com/wpdas/chain-db-rs) |
| TypeScript, JavaScript | [chain-db-ts](https://github.com/wpdas/chain-db-ts) |

## Temporary Server (test)

Use this server to test ChainDB while testing your application or while creating a new Client.

**Chain DB Test Server:** https://gull-dominant-mistakenly.ngrok-free.app. This server may not be always available.

**Rust using `chain-db-rs`:**

```rs
#[tokio::main]
async fn main() {
  // 1 - DB connection: server | db-name | user | password
  let db = ChainDB::connect(Some("https://gull-dominant-mistakenly.ngrok-free.app"), "test-db", "root", "1234");

  // 2 - Init a table
  let mut greetingTable = db.get_table("greeting", GreetingTable::new).await;
  println!("{:?}", greetingTable.table.greeting); // "Hi"

  // 3 - Mutate the table values and persist on chain
  greetingTable.table.greeting = String::from("Hello my dear!");
  greetingTable.persist().await; // Persist data on chain

  // 4 - See the most updated values of the table
  println!("{:?}", greetingTable.table.greeting); // "Hello my dear!"
}
```

**TypeScript / JavaScript using `chain-db-ts`:**

```ts
import { connect } from 'chain-db-ts'
import { GreetingTable } from './tables'

const main async () {
  // 1 - DB connection: server | db-name | user | password
  const db = connect("https://gull-dominant-mistakenly.ngrok-free.app", 'test-db', 'root', '1234')

  // 2 - Init a table
  const greetingTable = await db.get_table('greeting', new GreetingTable())
  console.log(greetingTable.table.greeting) // 'Hi'

  // 3 - Mutate the table values and persist on chain
  greetingTable.table.greeting = "Hello my dear!"
  await greetingTable.persist() // Data is persisted on the blockchain

  // 4 - See the most updated values of the table
  console.log(greetingTable.table.greeting) // 'Hello my dear!'
}
main()
```
