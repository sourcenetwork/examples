# DefraDB HTTP Rust Tutorials

This collection of Rust tutorials demonstrates how to interact with a running DefraDB instance using HTTP requests. Each tutorial covers different aspects of DefraDB functionality with comprehensive examples and explanations.

## Prerequisites

### 1. DefraDB Installation and Setup

You need a running DefraDB instance. There are several ways to set this up:

#### Option A: Using Docker (Recommended)
```bash
# Pull and run DefraDB
docker run -p 9181:9181 -p 9182:9182 ghcr.io/sourcenetwork/
# ussing --store=memory allow for easy cleanup in between runs
defradb:latest start --no-keyring --store=memory
```

#### Option B: From Source
```bash
# Clone and build DefraDB
git clone https://github.com/sourcenetwork/defradb.git
cd defradb
make install
defradb start --no-keyring --store=memory
```

#### Option C: Binary Release
Download the latest release from [DefraDB Releases](https://github.com/sourcenetwork/defradb/releases) and run:
```bash
defradb start --no-keyring --store=memory
```

### 2. DefraDB Node Configuration

For tutorials that require multiple nodes (like P2P networking), you'll need to run two DefraDB instances:

**Node 1 (Port 9181):**
```bash
defradb start --no-keyring --store=memory 
```

**Node 2 (Port 9182):**
```bash
defradb start --no-keyring --store=memory --rootdir ./node2 --url localhost:9182 --p2paddr /ip4/127.0.0.1/tcp/9172
```

### 3. Rust Development Environment

- **Rust**: Install Rust 1.70+ from [rustup.rs](https://rustup.rs/)
- **Cargo**: Comes with Rust installation

Verify installation:
```bash
rustc --version
cargo --version
```

## Project Structure

```
src/
├── schema_management.rs      # Schema creation, versioning, and JSON Patch operations
├── collection_operations.rs  # CRUD operations on documents
├── graphql_operations.rs     # GraphQL queries, mutations, and advanced features
├── p2p_networking.rs         # P2P replication and networking between nodes
├── backup_operations.rs      # Database export/import functionality
└── openapi.json             # DefraDB API specification reference
```

## Running the Tutorials

### 1. Install Dependencies

```bash
cargo build
```

### 2. Basic Single-Node Tutorials

These tutorials require only one DefraDB instance running on port 9181:

#### Schema Management Tutorial
```bash
cargo run --bin schema_management
```
**What it covers:**
- Creating and managing collection schemas
- Schema versioning with JSON Patch operations
- Setting default schema versions
- Field type definitions and validation

#### Collection Operations Tutorial
```bash
cargo run --bin collection_operations
```
**What it covers:**
- CRUD operations on documents
- SSE streaming for document IDs
- Filtering and batch operations
- Complex data types (JSON, arrays, DateTime)

#### GraphQL Operations Tutorial
```bash
cargo run --bin graphql_operations
```
**What it covers:**
- GraphQL queries and mutations
- Schema introspection
- Advanced filtering and sorting
- Aggregation operations
- Subscription handling

#### Backup Operations Tutorial
```bash
cargo run --bin backup_operations
```
**What it covers:**
- Database export/import functionality
- Backup strategies and best practices
- Data migration between instances

#### Indexing Operations Tutorial
```bash
cargo run --bin indexing_operations
```
**What it covers:**
- Creating and managing secondary indexes
- Query performance optimization
- Index types and use cases

### 3. Multi-Node Tutorials

These tutorials require two DefraDB instances running on ports 9181 and 9182:

#### P2P Networking Tutorial
```bash
# Make sure both nodes are running, then:
cargo run --bin p2p_networking
```
**What it covers:**
- Peer discovery and connection
- Collection replication between nodes
- Document synchronization strategies
- P2P network configuration

## Tutorial Features

### Comprehensive Examples
Each tutorial includes:
- **Setup code**: Automatic schema creation and test data
- **Error handling**: Robust error handling with meaningful messages
- **Best practices**: Recommended patterns and approaches
- **Real-world scenarios**: Practical examples you can adapt

### Educational Structure
- **Progressive complexity**: Tutorials start simple and build up
- **Detailed comments**: Every operation is explained
- **Output examples**: Shows what to expect when running
- **Troubleshooting**: Common issues and solutions

### Production-Ready Code
- **Proper error handling**: No unwrap() calls in production paths
- **Type safety**: Strongly typed request/response structures
- **Async/await**: Modern Rust async patterns
- **Resource cleanup**: Proper connection and resource management

## Troubleshooting

### Common Issues

#### DefraDB Connection Errors
```
Error: Connection refused (os error 61)
```
**Solution**: Ensure DefraDB is running on the expected port (9181 for most tutorials)

#### Schema Already Exists Errors
```
Error adding schema: schema already exists
```
**Solution**: This is expected behavior. The tutorials handle this gracefully and continue.

#### P2P Node Communication Issues
```
Error getting node 2 peer info: Request failed
```
**Solution**: Ensure both DefraDB nodes are running on ports 9181 and 9182.

#### Port Already in Use
```
Error: Address already in use (os error 48)
```
**Solution**: Stop any existing DefraDB instances or use different ports.

## API Reference

The `src/openapi.json` file contains the complete DefraDB API specification. Use it as a reference for:
- Available endpoints
- Request/response formats
- Parameter definitions
- Error codes

## Next Steps

After running these tutorials, you'll have a solid understanding of:
- DefraDB's HTTP API
- Rust async programming with DefraDB
- GraphQL operations and schema management
- P2P networking and data synchronization
- Transaction management and data consistency
- Access control and security

You can use these examples as a foundation for building your own DefraDB applications in Rust.

## Contributing

Found an issue or want to improve the tutorials? Please open an issue or submit a pull request to the DefraDB examples repository.