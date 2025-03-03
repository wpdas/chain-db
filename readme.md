# ChainDB

[English](#english) | [Português](README.pt-BR.md)

# English

ChainDB is a simple database that maintains a history of changes, allowing you to track how your data evolves over time.

## Features

- 📊 Complete history tracking of data changes
- 🔒 Secure data storage with AES-256-GCM encryption
- 🔑 Password-based key derivation
- 🌐 REST API for data manipulation
- 📚 Support for multiple databases
- 📁 Organized file structure with efficient data segmentation
- ⚡ Fast read and write operations
- 🔄 Automatic version control and data tracking

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

## Installation

Add ChainDB to your `Cargo.toml`:

```toml
[dependencies]
chain-db-v2 = "0.1.0"
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
}
```

## Testing the API

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

3. Change password:

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

4. Update table data:

```bash
curl -X POST http://localhost:2818/api/v1/table/greetings/update \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ=" \
  -d '{"data": {"greeting": "Hello, World!"}}'
```

5. Get current table data:

```bash
curl -X GET http://localhost:2818/api/v1/table/greetings \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ="
```

6. Persist new record:

```bash
curl -X POST http://localhost:2818/api/v1/table/greetings/persist \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ=" \
  -d '{"data": {"greeting": "New Record!"}}'
```

7. Get history:

```bash
curl -X GET http://localhost:2818/api/v1/table/greetings/history?limit=10 \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ="
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

## License / Licença

This project is licensed under the MIT License - see the LICENSE file for details.
