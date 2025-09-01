// DefraDB P2P Networking Tutorial
//
// This tutorial demonstrates how to set up peer-to-peer networking between DefraDB nodes.
// P2P functionality allows multiple DefraDB instances to synchronize data and form
// a distributed network. This tutorial uses two nodes:
// - Node 1: http://localhost:9181/api/v0
// - Node 2: http://localhost:9182/api/v0

use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::time::{Duration, sleep};

#[derive(Debug, Deserialize)]
struct DefraError {
    error: String,
}

// Peer information structure
#[derive(Debug, Deserialize, Serialize)]
struct PeerInfo {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Addresses")]
    addresses: Vec<String>,
}

// Replicator configuration
#[derive(Debug, Serialize)]
struct ReplicatorParams {
    #[serde(rename = "Info")]
    info: PeerInfo,
    #[serde(rename = "Collections")]
    collections: Vec<String>,
}

// Replicator status
#[derive(Debug, Deserialize)]
struct Replicator {
    #[serde(rename = "Info")]
    info: PeerInfo,
    #[serde(rename = "CollectionIDs")]
    collection_ids: Vec<String>,
    #[serde(rename = "Status")]
    status: u8,
    #[serde(rename = "LastStatusChange")]
    last_status_change: String,
}

// Document synchronization request
#[derive(Debug, Serialize)]
struct SyncDocumentsRequest {
    #[serde(rename = "collectionName")]
    collection_name: String,
    #[serde(rename = "docIDs")]
    doc_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<String>,
}

// Get peer information from a DefraDB node
async fn get_peer_info(client: &reqwest::Client, base_url: &str) -> Result<PeerInfo, String> {
    let url = format!("{}/p2p/info", base_url);

    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        let peer_info: PeerInfo = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse peer info: {}", e))?;
        Ok(peer_info)
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// Add a replicator to sync collections with another peer
async fn add_replicator(
    client: &reqwest::Client,
    base_url: &str,
    peer_info: PeerInfo,
    collections: Vec<String>,
) -> Result<(), String> {
    let url = format!("{}/p2p/replicators", base_url);
    let replicator_params = ReplicatorParams {
        info: peer_info,
        collections,
    };

    let response = match client.post(&url).json(&replicator_params).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        Ok(())
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// List all replicators on a node
async fn list_replicators(
    client: &reqwest::Client,
    base_url: &str,
) -> Result<Vec<Replicator>, String> {
    let url = format!("{}/p2p/replicators", base_url);

    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        let replicators: Vec<Replicator> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse replicators: {}", e))?;
        Ok(replicators)
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// Remove a replicator
async fn remove_replicator(
    client: &reqwest::Client,
    base_url: &str,
    peer_info: PeerInfo,
    collections: Vec<String>,
) -> Result<(), String> {
    let url = format!("{}/p2p/replicators", base_url);
    let replicator_params = ReplicatorParams {
        info: peer_info,
        collections,
    };

    let response = match client.delete(&url).json(&replicator_params).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        Ok(())
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// Add collections to peer synchronization
async fn add_peer_collections(
    client: &reqwest::Client,
    base_url: &str,
    collections: Vec<String>,
) -> Result<(), String> {
    let url = format!("{}/p2p/collections", base_url);

    let response = match client.post(&url).json(&collections).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        Ok(())
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// List collections being synchronized with peers
async fn list_peer_collections(
    client: &reqwest::Client,
    base_url: &str,
) -> Result<Vec<String>, String> {
    let url = format!("{}/p2p/collections", base_url);

    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        let collections: Vec<String> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse collections: {}", e))?;
        Ok(collections)
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// Remove collections from peer synchronization
async fn remove_peer_collections(
    client: &reqwest::Client,
    base_url: &str,
    collections: Vec<String>,
) -> Result<(), String> {
    let url = format!("{}/p2p/collections", base_url);

    let response = match client.delete(&url).json(&collections).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        Ok(())
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// Add specific documents to peer synchronization
async fn add_peer_documents(
    client: &reqwest::Client,
    base_url: &str,
    doc_ids: Vec<String>,
) -> Result<(), String> {
    let url = format!("{}/p2p/documents", base_url);

    let response = match client.post(&url).json(&doc_ids).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        Ok(())
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// List documents being synchronized with peers
async fn list_peer_documents(
    client: &reqwest::Client,
    base_url: &str,
) -> Result<Vec<String>, String> {
    let url = format!("{}/p2p/documents", base_url);

    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        let documents: Vec<String> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse documents: {}", e))?;
        Ok(documents)
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// Synchronize specific documents from the network
async fn sync_documents(
    client: &reqwest::Client,
    base_url: &str,
    collection_name: String,
    doc_ids: Vec<String>,
    timeout: Option<String>,
) -> Result<(), String> {
    let url = format!("{}/p2p/documents/sync", base_url);
    let sync_request = SyncDocumentsRequest {
        collection_name,
        doc_ids,
        timeout,
    };

    let response = match client.post(&url).json(&sync_request).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        Ok(())
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// Helper function to create test data
async fn create_test_user(
    client: &reqwest::Client,
    base_url: &str,
    user_data: serde_json::Value,
) -> Result<String, String> {
    let url = format!("{}/collections/User", base_url);

    let response = match client.post(&url).json(&user_data).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        Ok(response.text().await.unwrap())
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define our two DefraDB nodes
    let node1_url = "http://localhost:9181/api/v0";
    let node2_url = "http://localhost:9182/api/v0";
    let client = reqwest::Client::new();

    // Setup different schemas on both nodes for P2P demonstration
    println!("=== Setting up Multiple Schemas on Both Nodes ===");
    let schemas = vec![
        (
            "User",
            r#"
            type User {
                name: String
                email: String
                age: Int
            }
        "#,
        ),
        (
            "Product",
            r#"
            type Product {
                name: String
                price: Float
                category: String
            }
        "#,
        ),
        (
            "Message",
            r#"
            type Message {
                content: String
                sender: String
                timestamp: DateTime
            }
        "#,
        ),
    ];

    for (i, base_url) in [node1_url, node2_url].iter().enumerate() {
        println!("Setting up schemas on node {}", i + 1);
        for (collection_name, schema) in &schemas {
            let schema_url = format!("{}/schema", base_url);
            let response = client
                .post(&schema_url)
                .header("Content-Type", "text/plain")
                .body(*schema)
                .send()
                .await?;

            if response.status() == 200 || response.text().await?.contains("already exists") {
                println!("  {} schema ready on node {}", collection_name, i + 1);
            }
        }
    }

    // 1. Get peer information from both nodes
    println!("\n=== 1. Getting Peer Information ===");

    println!("Node 1 peer info:");
    let node1_peer_info = match get_peer_info(&client, node1_url).await {
        Ok(info) => {
            println!("  ID: {}", info.id);
            println!("  Addresses: {:?}", info.addresses);
            info
        }
        Err(e) => {
            eprintln!("Error getting node 1 peer info: {}", e);
            return Ok(());
        }
    };

    println!("\nNode 2 peer info:");
    let node2_peer_info = match get_peer_info(&client, node2_url).await {
        Ok(info) => {
            println!("  ID: {}", info.id);
            println!("  Addresses: {:?}", info.addresses);
            info
        }
        Err(e) => {
            eprintln!("Error getting node 2 peer info: {}", e);
            return Ok(());
        }
    };

    // 2. Set up replication for User collection (Node 1 -> Node 2)
    println!("\n=== 2. Setting up User Collection Replication (Node 1 -> Node 2) ===");
    let user_collections = vec!["User".to_string()];

    match add_replicator(
        &client,
        node1_url,
        node2_peer_info,
        user_collections.clone(),
    )
    .await
    {
        Ok(()) => println!("Successfully added User replicator on Node 1"),
        Err(e) => {
            if e.contains("already exists") {
                println!("User replicator already exists on Node 1");
            } else {
                eprintln!("Error adding User replicator to Node 1: {}", e);
            }
        }
    }

    // 3. Set up replication for Product collection (Node 2 -> Node 1)
    println!("\n=== 3. Setting up Product Collection Replication (Node 2 -> Node 1) ===");
    let product_collections = vec!["Product".to_string()];

    match add_replicator(
        &client,
        node2_url,
        node1_peer_info,
        product_collections.clone(),
    )
    .await
    {
        Ok(()) => println!("Successfully added Product replicator on Node 2"),
        Err(e) => {
            if e.contains("already exists") {
                println!("Product replicator already exists on Node 2");
            } else {
                eprintln!("Error adding Product replicator to Node 2: {}", e);
            }
        }
    }

    // 4. List replicators on both nodes
    println!("\n=== 4. Listing Replicators ===");

    for (i, base_url) in [node1_url, node2_url].iter().enumerate() {
        println!("Replicators on Node {}:", i + 1);
        match list_replicators(&client, base_url).await {
            Ok(replicators) => {
                if replicators.is_empty() {
                    println!("  No replicators configured");
                } else {
                    for replicator in replicators {
                        println!("  Peer ID: {}", replicator.info.id);
                        println!("  Collections: {:?}", replicator.collection_ids);
                        println!("  Status: {}", replicator.status);
                    }
                }
            }
            Err(e) => eprintln!("Error listing replicators on Node {}: {}", i + 1, e),
        }
    }

    // 5. Add different collections to peer synchronization on each node
    println!("\n=== 5. Managing Peer Collections (Different Collections per Node) ===");

    // Add Product collection to peer sync on Node 1
    println!("Adding Product collection to peer sync on Node 1");
    match add_peer_collections(&client, node1_url, vec!["Product".to_string()]).await {
        Ok(()) => println!("  Successfully added Product collection to peer sync on Node 1"),
        Err(e) => eprintln!("  Error adding Product peer collection: {}", e),
    }

    // Add Message collection to peer sync on Node 2
    println!("Adding Message collection to peer sync on Node 2");
    match add_peer_collections(&client, node2_url, vec!["Message".to_string()]).await {
        Ok(()) => println!("  Successfully added Message collection to peer sync on Node 2"),
        Err(e) => eprintln!("  Error adding Message peer collection: {}", e),
    }

    // List peer collections on both nodes
    for (i, base_url) in [node1_url, node2_url].iter().enumerate() {
        println!("Peer collections on Node {}:", i + 1);
        match list_peer_collections(&client, base_url).await {
            Ok(collections) => {
                for collection in collections {
                    println!("  - {}", collection);
                }
            }
            Err(e) => eprintln!("  Error listing peer collections: {}", e),
        }
    }

    // 6. Create test data for different collections on different nodes
    println!("\n=== 6. Creating Test Data on Different Nodes ===");

    // Create User data on Node 1 (will replicate to Node 2 via replicator)
    let test_user = serde_json::json!({
        "name": "Replicator Test User",
        "email": "replicator@example.com",
        "age": 29
    });

    let user_collection_url = format!("{}/collections/User", node1_url);
    let user_doc_id = match client
        .post(&user_collection_url)
        .json(&test_user)
        .send()
        .await
    {
        Ok(response) => {
            if response.status() == 200 {
                let result = response.text().await?;
                println!("Created User on Node 1: {}", result);
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                    parsed["_docID"].as_str().unwrap_or("").to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        }
        Err(e) => {
            eprintln!("Error creating User: {}", e);
            String::new()
        }
    };

    // Create Product data on Node 2 (will replicate to Node 1 via replicator)
    let test_product = serde_json::json!({
        "name": "P2P Laptop",
        "price": 1299.99,
        "category": "Electronics"
    });

    let product_collection_url = format!("{}/collections/Product", node2_url);
    let product_doc_id = match client
        .post(&product_collection_url)
        .json(&test_product)
        .send()
        .await
    {
        Ok(response) => {
            if response.status() == 200 {
                let result = response.text().await?;
                println!("Created Product on Node 2: {}", result);
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                    parsed["_docID"].as_str().unwrap_or("").to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        }
        Err(e) => {
            eprintln!("Error creating Product: {}", e);
            String::new()
        }
    };

    // Create Message data on both nodes (for document-level P2P sync)
    let test_message = serde_json::json!({
        "content": "Hello P2P World!",
        "sender": "system",
        "timestamp": "2024-01-15T10:30:00Z"
    });

    let message_collection_url = format!("{}/collections/Message", node1_url);
    let message_doc_id = match client
        .post(&message_collection_url)
        .json(&test_message)
        .send()
        .await
    {
        Ok(response) => {
            if response.status() == 200 {
                let result = response.text().await?;
                println!("Created Message on Node 1: {}", result);
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                    parsed["_docID"].as_str().unwrap_or("").to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        }
        Err(e) => {
            eprintln!("Error creating Message: {}", e);
            String::new()
        }
    };

    // 7. Wait for synchronization
    println!("\n=== 7. Waiting for Data Synchronization ===");
    println!("Waiting 3 seconds for data to sync between nodes...");
    sleep(Duration::from_secs(3)).await;

    // 8. Verify data synchronization across different collections
    println!("\n=== 8. Verifying Data Synchronization Across Collections ===");

    // Check User data on both nodes (should be replicated from Node 1 to Node 2)
    println!("Checking User collection replication (Node 1 -> Node 2):");
    for (i, base_url) in [node1_url, node2_url].iter().enumerate() {
        let graphql_url = format!("{}/graphql", base_url);
        let user_query = r#"
            query {
                User {
                    _docID
                    name
                    email
                }
            }
        "#;

        let gql_request = serde_json::json!({"query": user_query});

        match client.post(&graphql_url).json(&gql_request).send().await {
            Ok(response) => {
                if response.status() == 200 {
                    let result: serde_json::Value = response.json().await?;
                    if let Some(users) = result.get("data").and_then(|d| d.get("User")) {
                        println!(
                            "  Node {}: {} users",
                            i + 1,
                            users.as_array().unwrap_or(&vec![]).len()
                        );
                    }
                }
            }
            Err(e) => eprintln!("Error querying Users on Node {}: {}", i + 1, e),
        }
    }

    // Check Product data on both nodes (should be replicated from Node 2 to Node 1)
    println!("Checking Product collection replication (Node 2 -> Node 1):");
    for (i, base_url) in [node1_url, node2_url].iter().enumerate() {
        let graphql_url = format!("{}/graphql", base_url);
        let product_query = r#"
            query {
                Product {
                    _docID
                    name
                    price
                }
            }
        "#;

        let gql_request = serde_json::json!({"query": product_query});

        match client.post(&graphql_url).json(&gql_request).send().await {
            Ok(response) => {
                if response.status() == 200 {
                    let result: serde_json::Value = response.json().await?;
                    if let Some(products) = result.get("data").and_then(|d| d.get("Product")) {
                        println!(
                            "  Node {}: {} products",
                            i + 1,
                            products.as_array().unwrap_or(&vec![]).len()
                        );
                    }
                }
            }
            Err(e) => eprintln!("Error querying Products on Node {}: {}", i + 1, e),
        }
    }

    // 9. Manual document synchronization for Message collection
    if !message_doc_id.is_empty() {
        println!("\n=== 9. Manual Document Synchronization (Message Collection) ===");
        println!(
            "Manually syncing Message document {} from Node 1 to Node 2",
            message_doc_id
        );

        match sync_documents(
            &client,
            node2_url,
            "Message".to_string(),
            vec![message_doc_id.clone()],
            Some("30s".to_string()),
        )
        .await
        {
            Ok(()) => println!("Successfully synchronized Message document"),
            Err(e) => eprintln!("Error synchronizing Message document: {}", e),
        }

        // Verify the Message appeared on Node 2
        sleep(Duration::from_secs(2)).await;
        let graphql_url = format!("{}/graphql", node2_url);
        let message_query = r#"
            query {
                Message {
                    _docID
                    content
                    sender
                }
            }
        "#;

        let gql_request = serde_json::json!({"query": message_query});
        match client.post(&graphql_url).json(&gql_request).send().await {
            Ok(response) => {
                if response.status() == 200 {
                    let result: serde_json::Value = response.json().await?;
                    if let Some(messages) = result.get("data").and_then(|d| d.get("Message")) {
                        println!(
                            "Messages on Node 2 after sync: {}",
                            messages.as_array().unwrap_or(&vec![]).len()
                        );
                    }
                }
            }
            Err(e) => eprintln!("Error verifying Message sync: {}", e),
        }
    }

    // 10. Document-level peer management (using Message collection)
    println!("\n=== 10. Document-level Peer Management (Message Collection) ===");

    if !message_doc_id.is_empty() {
        // Add specific Message document to peer sync on Node 1
        match add_peer_documents(&client, node1_url, vec![message_doc_id.clone()]).await {
            Ok(()) => println!(
                "Added Message document {} to peer sync on Node 1",
                message_doc_id
            ),
            Err(e) => eprintln!("Error adding Message peer document: {}", e),
        }

        // List peer documents on Node 1
        match list_peer_documents(&client, node1_url).await {
            Ok(documents) => {
                println!("Peer documents on Node 1:");
                for doc in documents {
                    println!("  - {}", doc);
                }
            }
            Err(e) => eprintln!("Error listing peer documents on Node 1: {}", e),
        }
    }

    // Also demonstrate with Product document if available
    if !product_doc_id.is_empty() {
        // Add specific Product document to peer sync on Node 2
        match add_peer_documents(&client, node2_url, vec![product_doc_id.clone()]).await {
            Ok(()) => println!(
                "Added Product document {} to peer sync on Node 2",
                product_doc_id
            ),
            Err(e) => eprintln!("Error adding Product peer document: {}", e),
        }

        // List peer documents on Node 2
        match list_peer_documents(&client, node2_url).await {
            Ok(documents) => {
                println!("Peer documents on Node 2:");
                for doc in documents {
                    println!("  - {}", doc);
                }
            }
            Err(e) => eprintln!("Error listing peer documents on Node 2: {}", e),
        }
    }

    // 11. Testing different sync mechanisms across collections
    println!("\n=== 11. Testing Different Sync Mechanisms ===");

    // Test User collection sync (via replicator: Node 1 -> Node 2)
    let additional_user = serde_json::json!({
        "name": "Additional Sync User",
        "email": "sync@example.com",
        "age": 31
    });

    let user_url = format!("{}/collections/User", node1_url);
    match client.post(&user_url).json(&additional_user).send().await {
        Ok(response) => {
            if response.status() == 200 {
                println!("Created additional User on Node 1 (will auto-replicate to Node 2)");
            }
        }
        Err(e) => eprintln!("Error creating additional user: {}", e),
    }

    // Test Product collection sync (via replicator: Node 2 -> Node 1)
    let additional_product = serde_json::json!({
        "name": "P2P Mouse",
        "price": 29.99,
        "category": "Electronics"
    });

    let product_url = format!("{}/collections/Product", node2_url);
    match client
        .post(&product_url)
        .json(&additional_product)
        .send()
        .await
    {
        Ok(response) => {
            if response.status() == 200 {
                println!("Created additional Product on Node 2 (will auto-replicate to Node 1)");
            }
        }
        Err(e) => eprintln!("Error creating additional product: {}", e),
    }

    // Wait for automatic replication
    println!("Waiting 3 seconds for automatic replication...");
    sleep(Duration::from_secs(3)).await;

    // Verify replication worked
    println!("Verifying automatic replication:");

    // Check if additional User synced to Node 2
    let user_check_url = format!("{}/graphql", node2_url);
    let user_check_query = r#"
        query {
            User(filter: {name: {_eq: "Additional Sync User"}}) {
                _docID
                name
            }
        }
    "#;

    let gql_request = serde_json::json!({"query": user_check_query});
    match client.post(&user_check_url).json(&gql_request).send().await {
        Ok(response) => {
            if response.status() == 200 {
                let result: serde_json::Value = response.json().await?;
                if let Some(users) = result.get("data").and_then(|d| d.get("User")) {
                    if users.as_array().unwrap_or(&vec![]).is_empty() {
                        println!("  Additional User not yet synced to Node 2");
                    } else {
                        println!("  ✓ Additional User successfully replicated to Node 2");
                    }
                }
            }
        }
        Err(e) => eprintln!("Error checking User replication: {}", e),
    }

    // Check if additional Product synced to Node 1
    let product_check_url = format!("{}/graphql", node1_url);
    let product_check_query = r#"
        query {
            Product(filter: {name: {_eq: "P2P Mouse"}}) {
                _docID
                name
            }
        }
    "#;

    let gql_request = serde_json::json!({"query": product_check_query});
    match client
        .post(&product_check_url)
        .json(&gql_request)
        .send()
        .await
    {
        Ok(response) => {
            if response.status() == 200 {
                let result: serde_json::Value = response.json().await?;
                if let Some(products) = result.get("data").and_then(|d| d.get("Product")) {
                    if products.as_array().unwrap_or(&vec![]).is_empty() {
                        println!("  Additional Product not yet synced to Node 1");
                    } else {
                        println!("  ✓ Additional Product successfully replicated to Node 1");
                    }
                }
            }
        }
        Err(e) => eprintln!("Error checking Product replication: {}", e),
    }

    // 12. Clean up - Remove replicators and peer sync configurations (optional)
    println!("\n=== 12. Cleanup (Optional) ===");
    println!("To remove replicators and peer configurations, uncomment the following code:");

    /*
    // Remove User replicator from Node 1
    match remove_replicator(&client, node1_url, node2_peer_info.clone(), user_collections).await {
        Ok(()) => println!("Removed User replicator from Node 1"),
        Err(e) => eprintln!("Error removing User replicator: {}", e),
    }

    // Remove Product replicator from Node 2
    match remove_replicator(&client, node2_url, node1_peer_info.clone(), product_collections).await {
        Ok(()) => println!("Removed Product replicator from Node 2"),
        Err(e) => eprintln!("Error removing Product replicator: {}", e),
    }

    // Remove peer collections
    match remove_peer_collections(&client, node1_url, vec!["Product".to_string()]).await {
        Ok(()) => println!("Removed Product from peer collections on Node 1"),
        Err(e) => eprintln!("Error removing peer collections: {}", e),
    }

    match remove_peer_collections(&client, node2_url, vec!["Message".to_string()]).await {
        Ok(()) => println!("Removed Message from peer collections on Node 2"),
        Err(e) => eprintln!("Error removing peer collections: {}", e),
    }
    */

    println!("\n=== P2P Networking Tutorial Complete ===");
    println!("You've learned how to:");
    println!("- Get peer information from DefraDB nodes");
    println!(
        "- Set up replicators for different collections (User: Node1->Node2, Product: Node2->Node1)"
    );
    println!(
        "- Configure collection-level peer synchronization (Product on Node1, Message on Node2)"
    );
    println!("- Manage document-level peer sync (Message documents)");
    println!("- Test automatic replication across different collections");
    println!("- Manually trigger document synchronization");
    println!("- Verify data consistency across nodes and collections");
    println!("- Clean up replication configuration");

    println!("\nP2P Synchronization Summary:");
    println!("- Replicators: Auto-sync entire collections between specific nodes");
    println!("- Peer Collections: Configure which collections participate in P2P network");
    println!("- Peer Documents: Fine-grained control over individual document sync");
    println!("- Each mechanism works independently and can target different collections");
    Ok(())
}
