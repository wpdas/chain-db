<div align="center">

  <h1><code>Chain DB</code></h1>

  <p>
    <strong>Story-driven Database</strong>
  </p>
  
  <h3>
    <!-- <a href="https://borsh.io">Website</a> -->
    <a href="https://github.com/wpdas/chain-db">About</a>
    <span> | </span>
    <a href="https://github.com/wpdas/chain-db#librariesclient">Libraries</a>
  </h3>
</div>

Chain DB is a Story-driven database. This new type of system uses some features used in blockchain technology. Each change generates a transaction that is saved in a block. The network works centrally, so persistent data is not decentralized.

The way data is read resembles the format of blockchains as well. The generated blocks are immutable. New data or updated data are saved in newer blocks, in this way, it is possible to recover all data from a table throughout its life.

We are just at the beginning of the implementation of this database.

Below are links to libraries (client) to be used with your preferred programming language. You can download the latest version of the database in the release section.

## Temporary Server (test)

Use this server to test ChainDB while testing your application or while creating a new Client.

**Chain DB Test Server:** https://gull-dominant-mistakenly.ngrok-free.app

E.g. (rust using `chain-db-rs`):

```rs
#[tokio::main]
async fn main() {
  // 1 - DB connection: server | db-name | user | password
  let db = ChainDB::connect(Some("https://gull-dominant-mistakenly.ngrok-free.app"), "my-db", "root", "1234");

  // 2 - Init a table
  let mut greeting = db.get_table("greeting", GreetingTable::new).await;
  println!("Current greeting: {:?}", greeting.table.greeting); // Hi

  // 3 - Mutate the table values and persist on chain
  greeting.table.set_greeting(String::from("Hello!"));
  greeting.persist().await; // Persist data on chain

  // 4 - See the most updated values of the table
  println!("Current greeting: {:?}", greeting.table.greeting); // Hello!
}
```

## Libraries (client)

| Platform | Repository                                          |
| -------- | --------------------------------------------------- |
| Rust     | [chain-db-rs](https://github.com/wpdas/chain-db-rs) |
