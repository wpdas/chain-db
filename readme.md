<div align="center">

  <h1><code>Chain DB</code></h1>

  <p>
    <strong>History-driven Database</strong>
  </p>
  
  <h3>
    <!-- <a href="https://borsh.io">Website</a> -->
    <a href="https://github.com/wpdas/chain-db">About</a>
    <span> | </span>
    <a href="https://github.com/wpdas/chain-db#download">Download</a>
    <span> | </span>
    <a href="https://github.com/wpdas/chain-db#libraries-client">Libraries</a>
    <span> | </span>
    <a href="https://github.com/wpdas/chain-db-cli">CLI</a>
  </h3>
</div>

ChainDB is a simple database that maintains a history of changes, allowing you to track how your data evolves over time.

## Download

Go to [Releases](https://github.com/wpdas/chain-db/releases) page and download the binary according to the desired architecture.

| Binary                                                                                                              | OS             | Devices                              |
| ------------------------------------------------------------------------------------------------------------------- | -------------- | ------------------------------------ |
| [chain-db-linux-amd64](https://github.com/wpdas/chain-db/releases/download/v1.1.0/chain-db-linux-amd64)             | Linux (x86_64) | Running Ubuntu or similar            |
| [chain-db-linux-arm64](https://github.com/wpdas/chain-db/releases/download/v1.1.0/chain-db-linux-arm64)             | Linux (ARM64)  | Raspberry Pi3, Raspberry Pi4, Others |
| [chain-db-linux-armv7](https://github.com/wpdas/chain-db/releases/download/v1.1.0/chain-db-linux-armv7)             | Linux (ARMv7)  | Raspberry Pi2, Raspberry Pi3, Others |
| [chain-db-macos-amd64](https://github.com/wpdas/chain-db/releases/download/v1.1.0/chain-db-macos-amd64)             | macOS 11+      | Apple                                |
| [chain-db-windows-amd64.exe](https://github.com/wpdas/chain-db/releases/download/v1.1.0/chain-db-windows-amd64.exe) | Windows        | Windows x86_64                       |

## Client Libraries (backend)

Below are links to libraries (for backend) to be used with your preferred programming language.

| Platform               | Repository                                          |
| ---------------------- | --------------------------------------------------- |
| TypeScript, JavaScript | [chain-db-ts](https://github.com/wpdas/chain-db-ts) |

## Command-Line Interface (CLI)

ChainDB comes with a powerful command-line interface that allows you to interact with your databases directly from the terminal. You can create and manage databases, manipulate tables, and perform advanced queries without writing any code.

Install it globally via npm:

```bash
# Min Node version: 20.18.3
npm install -g chain-db-cli
```

Learn more at [chain-db-cli](https://github.com/wpdas/chain-db-cli).

## Temporary Server (test)

Use this server to test ChainDB while testing your application or while creating a new Client Library.

**Chain DB Test Server:** https://gull-dominant-mistakenly.ngrok-free.app.

You can use it instead of `http://localhost:2818`. This server may not be always available.

## Example Apps

[Chat App](https://github.com/wpdas/chat-app-chaindb-nextjs): Real-time Chat App built using ChainDB and ChainDB TS Client.

## Features

- 📊 Complete history tracking of data changes
- 🔒 Secure data storage with AES-256-GCM encryption
- 🔑 Password-based key derivation
- 🌐 REST API for data manipulation
- 📚 Support for multiple databases
- 📁 Organized file structure with efficient data segmentation
- ⚡ Fast read and write operations
- 🔄 Automatic version control and data tracking
- 🔍 Advanced search capabilities with multiple comparison operators (Eq, Ne, Gt, Ge, Lt, Le, Contains, StartsWith, EndsWith)
- 🔎 Simple and complex query support for efficient data retrieval
- 📡 Real-time event system via WebSockets for instant data synchronization
- 🔔 Event notifications for table updates and new record persistence
- 🔌 Automatic WebSocket subscriptions for seamless client integration
- 🖥️ Command-line interface (CLI) for easy database management and operations via terminal - [chain-db-cli](https://github.com/wpdas/chain-db-cli)

## Security and Encryption

ChainDB uses a robust encryption system to protect your data:

1. **Key Derivation System**

   - Each password generates a unique encryption key using SHA-256
   - The same password always generates the same key
   - Different passwords generate different keys
   - The encryption key is never stored, only derived when needed

2. **Data Encryption**

   - All data is encrypted using AES-256-GCM
   - Each record is encrypted individually
   - A unique nonce is used for each encryption operation
   - The nonce is stored alongside the encrypted data

3. **Password Management**

   - Passwords are never stored in plain text
   - Changing password requires re-encrypting all data
   - Password change is an atomic operation (all or nothing)
   - If a password is lost, data cannot be recovered

4. **Data Storage Process**
   - When saving data:
     1. Data is converted to JSON format (human-readable)
     2. JSON is encrypted using the derived key (AES-256-GCM)
     3. Encrypted data is saved to .cdb files
   - When reading data:
     1. Encrypted data is read from .cdb files
     2. Data is decrypted using the derived key
     3. JSON is parsed back into the data structure

<!-- ## Installation

Add ChainDB to your `Cargo.toml`:

```toml
[dependencies]
chain-db = "1.1.0"
```

## Quick Start

Here's a basic example of how to use ChainDB:

```rust
use chain_db_v2::{ChainDB};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GreetingTable {
    greeting: String,
}

impl GreetingTable {
    fn new() -> Self {
        GreetingTable {
            greeting: String::from("Hi"),
        }
    }
}

#[tokio::main]
async fn main() {
    // 1. Connect to database
    let connection = ChainDB::connect(
        "test-db",    // Database name
        "root",       // User
        "1234",       // Password
    ).await;
    let db = &connection.db;

    // 2. Create/Get a table
    let mut greeting_table = db.get_table("greeting", GreetingTable::new).await;

    // 3. Read current value
    println!("{:?}", greeting_table.table.greeting);

    // 4. Update last record
    greeting_table.table.greeting = String::from("Hello World!");
    greeting_table.update_data().await;

    // 5. Create new record
    greeting_table.table.greeting = String::from("Hey!");
    greeting_table.persist_data().await;

    // 6. Get history (last 50 records)
    let history = greeting_table.get_data_history(50).await;
    println!("{:?}", history);
} -->

## Testing the API (for this program development process)

1. Start the server:

```bash
cargo run
```

The server will start at `http://localhost:2818`

### Authentication

All API endpoints (except database creation) require authentication using Basic Auth. The authentication header should be constructed as follows:

1. Create a string in the format: `database_name:username:password`
   - Example: `test_db:root:1234`
2. Encode this string in Base64
   - Example: `dGVzdF9kYjpyb290OjEyMzQ=`
3. Add the prefix "Basic " to the encoded string
   - Final header: `Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ=`

You can generate the Base64 string using various tools:

```bash
# Using command line
echo -n "test_db:root:1234" | base64

# Using Python
python3 -c "import base64; print(base64.b64encode('test_db:root:1234'.encode()).decode())"
```

### API Endpoints

1. Create a database:

```bash
curl -X POST http://localhost:2818/api/v1/database/create \
  -H "Content-Type: application/json" \
  -d '{"name": "test_db", "user": "root", "password": "1234"}'
```

2. Connect to database:

```bash
curl -X POST http://localhost:2818/api/v1/database/connect \
  -H "Content-Type: application/json" \
  -d '{"name": "test_db", "user": "root", "password": "1234"}'
```

The response will include an authentication token that should be used in subsequent API calls:

```json
{
  "success": true,
  "data": "Basic dGVzdF9kYjpyb290OjEyMzQ="
}
```

Use this token in the Authorization header for all subsequent requests:

```bash
Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ=
```

3. List all tables in the database:

```bash
curl -X GET http://localhost:2818/api/v1/tables \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ="
```

Response example:

```json
{
  "success": true,
  "message": null,
  "data": ["users", "products", "orders"]
}
```

4. Change password:

```bash
curl -X POST http://localhost:2818/api/v1/database/change-password \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test_db",
    "user": "root",
    "old_password": "1234",
    "new_password": "new_password"
  }'
```

**Important**: When changing password, all data needs to be re-encrypted with the new key. This process:

- May take longer for larger databases
- Is atomic (either all data is re-encrypted or none)
- Cannot be interrupted safely
- Should be done during low-usage periods
- Creates an automatic backup of all data before making changes (in a folder named `bkp_YYYYMMDD_HHMMSS`)
- Verifies available disk space before starting (requires at least 2x the current database size)
- Rolls back to the original state if any error occurs during the process

5. Update table data:

```bash
# Update a record by doc_id (required)
curl -X POST http://localhost:2818/api/v1/table/greetings/update \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ=" \
  -d '{"data": {"greeting": "Updated specific record!"}, "doc_id": "550e8400-e29b-41d4-a716-446655440000"}'
```

6. Get current table data:

```bash
curl -X GET http://localhost:2818/api/v1/table/greetings \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ="
```

7. Persist new record:

```bash
curl -X POST http://localhost:2818/api/v1/table/greetings/persist \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ=" \
  -d '{"data": {"greeting": "New Record!"}}'
```

The response will include the newly created record with its system-generated `doc_id`:

```json
{
  "success": true,
  "message": null,
  "data": {
    "greeting": "New Record!",
    "doc_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

8. Get history:

```bash
curl -X GET http://localhost:2818/api/v1/table/greetings/history?limit=10 \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ="
```

9. Get document by doc_id:

```bash
curl -X GET http://localhost:2818/api/v1/table/greetings/doc/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ="
```

Response:

```json
{
  "success": true,
  "message": null,
  "data": {
    "greeting": "Hello!",
    "doc_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

## How It Works

1. **Database Creation**:

   - Creates a folder with the database name
   - Stores settings in `config.cdb`
   - Securely stores credentials

2. **Table Management**:

   - Each table has its own directory
   - Data is stored in blocks of 1000 records per file
   - Automatic file management and indexing

3. **Operations**:
   - `update_data()`: Updates the last record
   - `persist_data()`: Creates a new record
   - `get_data_history(n)`: Retrieves the last N records

## Security

- Passwords are stored using bcrypt hashing (industry standard for password security)
- Data is serialized using Borsh
- Authentication required for all operations

## File Structure

```
your-database/
├── config.cdb             # Encrypted database configuration
├── table1/               # First table directory
│   ├── metadata.cdb     # Encrypted table metadata
│   ├── data_0.cdb      # First data block
│   ├── data_1.cdb      # Second data block
│   └── ...
└── table2/              # Second table directory
    ├── metadata.cdb
    └── ...
```

## New Search Features

### Data Structure

It's important to understand that the data in the table is stored with the following structure:

```json
{
  "data": {
    "field1": "value1",
    "field2": "value2",
    ...
  }
}
```

Therefore, when performing searches, the criteria must match the fields within the `data` object.

### Simple Search (findWhere)

The `POST /table/<table_name>/find` route allows you to search for records based on simple equality criteria.

**Request Example:**

```json
{
  "criteria": {
    "name": "John",
    "age": 30
  },
  "limit": 10,
  "reverse": true
}
```

In this example, the search will look for records where `data.name` equals "John" AND `data.age` equals 30.

You can also search for records by their `doc_id`:

```json
{
  "criteria": {
    "doc_id": "550e8400-e29b-41d4-a716-446655440000"
  },
  "limit": 10,
  "reverse": true
}
```

**Parameters:**

- `criteria`: An object containing the search criteria, where the keys are the field names within the `data` object and the values are the expected values for those fields. You can also search for fields in the root object, such as `doc_id`.
- `limit` (optional): Maximum number of records to be returned.
- `reverse` (optional): If true, search from the most recent record to the oldest (default: true).

**Response Example:**

```json
{
  "success": true,
  "message": null,
  "data": [
    {
      "name": "John",
      "age": 30,
      "city": "New York"
    },
    {
      "name": "John",
      "age": 30,
      "city": "Los Angeles"
    }
  ]
}
```

### Advanced Search (findWhereAdvanced)

The `POST /table/<table_name>/find-advanced` route allows you to search for records based on more complex comparison criteria.

**Request Example:**

```json
{
  "criteria": [
    {
      "field": "age",
      "operator": "Gt",
      "value": 25
    },
    {
      "field": "name",
      "operator": "Contains",
      "value": "Smith"
    },
    {
      "field": "city",
      "operator": "Eq",
      "value": "New York"
    }
  ],
  "limit": 20,
  "reverse": true
}
```

In this example, the search will look for records where:

- `data.age` is greater than 25 AND
- `data.name` contains "Smith" AND
- `data.city` equals "New York"

You can also search for records by their `doc_id`:

```json
{
  "criteria": [
    {
      "field": "doc_id",
      "operator": "Eq",
      "value": "550e8400-e29b-41d4-a716-446655440000"
    }
  ],
  "limit": 10,
  "reverse": true
}
```

**Parameters:**

- `criteria`: An array of objects containing the search criteria, where each object has:
  - `field`: The name of the field within the `data` object to be compared. You can also search for fields in the root object, such as `doc_id`.
  - `operator`: The comparison operator to be used.
  - `value`: The value to be compared.

**Available Comparison Operators:**

- `Eq`: Equal to (==)
- `Ne`: Not equal to (!=)
- `Gt`: Greater than (>)
- `Ge`: Greater than or equal to (>=)
- `Lt`: Less than (<)
- `Le`: Less than or equal to (<=)
- `Contains`: Contains (for strings and arrays)
- `StartsWith`: Starts with (for strings)
- `EndsWith`: Ends with (for strings)

**Response Example:**

```json
{
  "success": true,
  "message": null,
  "data": [
    {
      "name": "Joseph Smith",
      "age": 35,
      "city": "New York"
    },
    {
      "name": "Mary Smith",
      "age": 28,
      "city": "New York"
    }
  ]
}
```

## Real-Time Event System

ChainDB now includes a real-time event system that allows clients to receive automatic notifications when changes occur in tables. This is useful for keeping user interfaces synchronized with the database without the need for constant polling.

### Event Types

The following event types are available:

- `TableUpdate`: Emitted when a record is updated using the `update()` method
- `TablePersist`: Emitted when a new record is persisted using the `persist()` method

### Receiving Real-Time Events

To receive real-time events, the client only needs to establish a WebSocket connection with the authentication token. There is no need to send an explicit subscription message.

#### 1. Establishing WebSocket Connection

```javascript
// JavaScript example
const token = "Basic dGVzdF9kYjpyb290OjEyMzQ="; // Authentication token
const ws = new WebSocket(`ws://localhost:2818/api/v1/events`, {
  headers: {
    Authorization: token,
  },
});

// On connect
ws.onopen = () => {
  console.log("WebSocket connection established");
};
```

#### 2. Receiving Events

```javascript
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);

  // Check if it's a connection confirmation
  if (data.status === "connected") {
    console.log("Successfully connected to database:", data.database);
    if (data.table) {
      console.log("Table:", data.table);
    }
    return;
  }

  // Process the event
  console.log("Event received:", data);

  // Example of processing based on event type
  switch (data.event_type) {
    case "TableUpdate":
      console.log("Table updated:", data.table);
      console.log("New data:", data.data);
      // Update the user interface
      break;
    case "TablePersist":
      console.log("New record persisted:", data.table);
      console.log("Data:", data.data);
      // Add the new record to the user interface
      break;
  }
};
```

#### 3. Listing Available Event Types

```bash
curl -X GET http://localhost:2818/api/v1/events/types \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ="
```

Response:

```json
{
  "success": true,
  "message": null,
  "data": ["TableUpdate", "TablePersist"]
}
```

### Event Structure

Events have the following structure:

```json
{
  "event_type": "TableUpdate",
  "database": "my_database",
  "table": "myTable",
  "data": {
    "field1": "value1",
    "field2": "value2"
  },
  "timestamp": 1621234567
}
```

- `event_type`: Type of event (TableUpdate, TablePersist)
- `database`: Database name
- `table`: Table name
- `data`: Data associated with the event (the internal data of the record)
- `timestamp`: Event timestamp (seconds since epoch)

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Document IDs and Record Management

ChainDB now assigns a unique document ID (`doc_id`) to each record when it is created. This provides several benefits:

1. **Unique Identification**: Each record has a UUID that uniquely identifies it throughout its lifecycle.

2. **Targeted Updates**: You can update specific records by referencing their `doc_id`, not just the most recent one.

3. **Improved Data Management**: The `doc_id` makes it easier to track and reference specific records in your application logic.

When you persist a new record, a `doc_id` is automatically generated and included in the record:

```json
{
  "greeting": "Hello!",
  "doc_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Important**: The `doc_id` is managed by the system and cannot be set or modified by the user. If you include a `doc_id` in your data when creating or updating records, it will be ignored and replaced with a system-generated value (for new records) or the existing value (for updates).

### Updating Specific Records

To update a record in the database, you must specify its `doc_id`. The `doc_id` is now a required parameter for all update operations.

**Important**: When updating a record, the new data completely replaces the existing data. There is no merging of properties. If you want to preserve existing properties, you must include them in your update request.

For example, if your record has:

```json
{
  "name": "John",
  "age": 30,
  "city": "New York",
  "doc_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

And you update it with:

```json
{
  "data": {
    "age": 31
  },
  "doc_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

The resulting record will be:

```json
{
  "age": 31,
  "doc_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

The `name` and `city` properties are lost because the update completely replaces the data.

Note that updating older records may be more computationally expensive, as the system needs to search through the history to find the specified record. The performance impact depends on how far back in history the record is located.
