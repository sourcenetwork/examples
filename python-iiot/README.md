# DefraDB Python IIoT Example

This example demonstrates how to verify signed sensor data stored in DefraDB using Python. The application polls DefraDB for sensor events and validates the cryptographic signatures to ensure data integrity.

## Overview

This example consists of two main components:

### Data Ingestion (`ingest_data.py`)
1. **Reads serial data** - Connects to a serial port to receive sensor readings from hardware devices
2. **Parses JSON** - Decodes JSON-formatted sensor events containing deviceId, timestamp, prediction, and probability
3. **Stores in DefraDB** - Uses GraphQL mutations to create signed sensor events in DefraDB
4. **Automatic signing** - DefraDB automatically signs each document using the node's private key

### Data Verification (`verifier.py`)
1. **Polls DefraDB** - Continuously queries for new sensor events using GraphQL
2. **Retrieves commit data** - Fetches the latest commits for each sensor event to get signature information
3. **Validates signatures** - Converts base64-encoded public keys to hex format and verifies block signatures via DefraDB's verification endpoint
4. **Reports status** - Displays verification results for each sensor reading with timestamp, device ID, prediction, and probability

## Key Features

- **Cryptographic verification** - Validates data integrity using secp256k1 signatures
- **GraphQL integration** - Uses DefraDB's GraphQL API for data retrieval

## Configuration

### Data Ingestion
Set environment variables for `ingest_data.py`:
- `SERIAL_PORT` - Serial port for sensor data (default: /dev/ttyACM0)
- `BAUD` - Serial baud rate (default: 115200)
- `GQL_URL` - DefraDB GraphQL endpoint (default: http://localhost:9181/api/v0/graphql)

### Data Verification
Set environment variables for `verifier.py`:
- `GQL_URL` - DefraDB GraphQL endpoint (default: http://localhost:9182/api/v0/graphql)
- `VERIFY_URL` - DefraDB signature verification endpoint (default: http://localhost:9182/api/v0/block/verify-signature)

