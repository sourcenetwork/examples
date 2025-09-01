// DefraDB Collection Operations Tutorial
//
// This tutorial demonstrates how to perform CRUD operations on collections in DefraDB.
// Collections are where your actual data documents are stored.
// This covers creating, reading, updating, and deleting documents using the REST API.

use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Deserialize)]
struct DefraError {
    error: String,
}

// Document ID result from SSE stream
#[derive(Debug, Deserialize)]
struct DocIDResult {
    #[serde(rename = "docID")]
    doc_id: String,
    error: String,
}

// Request/Response structures for collection operations
#[derive(Debug, Serialize, Deserialize)]
struct User {
    #[serde(rename = "_docID", skip_serializing_if = "Option::is_none")]
    doc_id: Option<String>,
    name: String,
    email: String,
    age: i32,
}

#[derive(Debug, Deserialize)]
struct DeleteResult {
    #[serde(rename = "Count")]
    count: i64,
    #[serde(rename = "DocIDs")]
    doc_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateResult {
    #[serde(rename = "Count")]
    count: i64,
    #[serde(rename = "DocIDs")]
    doc_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CollectionUpdate {
    filter: serde_json::Value,
    updater: String,
}

#[derive(Debug, Serialize)]
struct CollectionDelete {
    filter: serde_json::Value,
}

// Create a single document in a collection
async fn create_document(
    client: &reqwest::Client,
    base_url: &str,
    collection_name: &str,
    document: &serde_json::Value,
) -> Result<(), String> {
    let url = format!("{}/collections/{}", base_url, collection_name);

    let response = match client.post(&url).json(document).send().await {
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

// Create multiple documents in a collection
async fn create_documents(
    client: &reqwest::Client,
    base_url: &str,
    collection_name: &str,
    documents: &Vec<serde_json::Value>,
) -> Result<String, String> {
    let url = format!("{}/collections/{}", base_url, collection_name);

    let response = match client.post(&url).json(documents).send().await {
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

// Get a specific document by its docID
async fn get_document(
    client: &reqwest::Client,
    base_url: &str,
    collection_name: &str,
    doc_id: &str,
) -> Result<serde_json::Value, String> {
    let url = format!("{}/collections/{}/{}", base_url, collection_name, doc_id);

    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        let document: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse document: {}", e))?;
        Ok(document)
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// Get all document IDs in a collection (SSE stream)
// This endpoint returns document IDs as Server-Sent Events (SSE) - each document ID is sent as a separate event
async fn get_document_ids(
    client: &reqwest::Client,
    base_url: &str,
    collection_name: &str,
) -> Result<Vec<String>, String> {
    let url = format!("{}/collections/{}", base_url, collection_name);

    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read SSE response: {}", e))?;

    if status == 200 {
        // Parse SSE format: each line contains a JSON object with docID and error fields
        let mut doc_ids = Vec::new();
        let mut errors = Vec::new();

        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("data: ") {
                // Standard SSE format: "data: {json_object}"
                let json_data = line.strip_prefix("data: ").unwrap_or("").trim();
                if !json_data.is_empty() {
                    match serde_json::from_str::<DocIDResult>(json_data) {
                        Ok(result) => {
                            if result.error.is_empty() {
                                doc_ids.push(result.doc_id);
                            } else {
                                errors.push(format!("DocID {}: {}", result.doc_id, result.error));
                            }
                        }
                        Err(e) => {
                            // If parsing fails, it might be a plain string (fallback)
                            if json_data.starts_with('"') && json_data.ends_with('"') {
                                let doc_id = json_data.trim_matches('"');
                                doc_ids.push(doc_id.to_string());
                            } else {
                                eprintln!("Failed to parse SSE data '{}': {}", json_data, e);
                            }
                        }
                    }
                }
            }
            // Ignore other SSE control lines like "event:", "id:", "retry:", or comments ":"
        }

        // Return error if there were any errors, otherwise return the document IDs
        if !errors.is_empty() {
            Err(format!(
                "Errors retrieving document IDs: {}",
                errors.join("; ")
            ))
        } else {
            Ok(doc_ids)
        }
    } else {
        if let Ok(error) = serde_json::from_str::<DefraError>(&text) {
            Err(error.error)
        } else {
            Err(format!("Request failed with status: {} - {}", status, text))
        }
    }
}

// Update a specific document by docID
async fn update_document(
    client: &reqwest::Client,
    base_url: &str,
    collection_name: &str,
    doc_id: &str,
    updates: &serde_json::Value,
) -> Result<(), String> {
    let url = format!("{}/collections/{}/{}", base_url, collection_name, doc_id);

    let response = match client.patch(&url).json(updates).send().await {
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

// Update documents using a filter
async fn update_documents_with_filter(
    client: &reqwest::Client,
    base_url: &str,
    collection_name: &str,
    filter: serde_json::Value,
    updater: String,
) -> Result<UpdateResult, String> {
    let url = format!("{}/collections/{}", base_url, collection_name);
    let update_request = CollectionUpdate { filter, updater };

    let response = match client.patch(&url).json(&update_request).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        let result: UpdateResult = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse result: {}", e))?;
        Ok(result)
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// Delete a specific document by docID
async fn delete_document(
    client: &reqwest::Client,
    base_url: &str,
    collection_name: &str,
    doc_id: &str,
) -> Result<(), String> {
    let url = format!("{}/collections/{}/{}", base_url, collection_name, doc_id);

    let response = match client.delete(&url).send().await {
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

// Delete documents using a filter
async fn delete_documents_with_filter(
    client: &reqwest::Client,
    base_url: &str,
    collection_name: &str,
    filter: serde_json::Value,
) -> Result<DeleteResult, String> {
    let url = format!("{}/collections/{}", base_url, collection_name);
    let delete_request = CollectionDelete { filter };

    let response = match client.delete(&url).json(&delete_request).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        let result: DeleteResult = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse result: {}", e))?;
        Ok(result)
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = "http://localhost:9181/api/v0";
    let client = reqwest::Client::new();

    // First, ensure we have a User schema
    println!("=== Setting up User Schema ===");
    let user_schema = r#"
        type User {
            name: String
            email: String
            age: Int
        }
    "#;

    // Add schema (will skip if already exists)
    let schema_url = format!("{}/schema", base_url);
    let _ = client
        .post(&schema_url)
        .header("Content-Type", "text/plain")
        .body(user_schema)
        .send()
        .await;

    // 1. Create a single user document
    println!("\n=== Creating Single User ===");
    let user1 = serde_json::json!({
        "name": "Alice Smith",
        "email": "alice@example.com",
        "age": 30
    });

    match create_document(&client, base_url, "User", &user1).await {
        Ok(()) => println!("Created user"),
        Err(e) => eprintln!("Error creating user: {}", e),
    }

    // 2. Create multiple users at once
    println!("\n=== Creating Multiple Users ===");
    let users = vec![
        serde_json::json!({
            "name": "Bob Johnson",
            "email": "bob@example.com",
            "age": 25
        }),
        serde_json::json!({
            "name": "Carol Williams",
            "email": "carol@example.com",
            "age": 35
        }),
        serde_json::json!({
            "name": "David Brown",
            "email": "david@example.com",
            "age": 28
        }),
    ];

    match create_documents(&client, base_url, "User", &users).await {
        Ok(result) => println!("Created users: {}", result),
        Err(e) => eprintln!("Error creating users: {}", e),
    }

    // 3. Get all document ids
    println!("\n=== Getting All User Document ids ===");
    match get_document_ids(&client, base_url, "User").await {
        Ok(ids) => {
            println!("Found {} user documents:", ids.len());
            for key in &ids {
                println!("  Document ID: {}", key);
            }

            // 4. Get a specific document
            if !ids.is_empty() {
                println!("\n=== Getting Specific User Document ===");
                match get_document(&client, base_url, "User", &ids[0]).await {
                    Ok(document) => {
                        println!("Retrieved document:");
                        println!("{}", serde_json::to_string_pretty(&document).unwrap());
                    }
                    Err(e) => eprintln!("Error getting document: {}", e),
                }

                // 5. Update the specific document
                println!("\n=== Updating Specific User Document ===");
                let updates = serde_json::json!({
                    "age": 31
                });

                match update_document(&client, base_url, "User", &ids[0], &updates).await {
                    Ok(()) => println!("Successfully updated document {}", ids[0]),
                    Err(e) => eprintln!("Error updating document: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error getting document keys: {}", e),
    }

    // 6. Update documents using a filter
    println!("\n=== Updating Users with Filter ===");
    let filter = serde_json::json!({
        "age": {"_gt": 30}  // Update users older than 30
    });
    let updater = r#"{"age": 40}"#.to_string(); // Set their age to 40

    match update_documents_with_filter(&client, base_url, "User", filter, updater).await {
        Ok(result) => {
            println!("Updated {} documents", result.count);
            for doc_id in &result.doc_ids {
                println!("  Updated document: {}", doc_id);
            }
        }
        Err(e) => eprintln!("Error updating with filter: {}", e),
    }

    // 7. Delete documents using a filter
    println!("\n=== Deleting Users with Filter ===");
    let delete_filter = serde_json::json!({
        "age": {"_eq": 25}  // Delete users with age 25
    });

    match delete_documents_with_filter(&client, base_url, "User", delete_filter).await {
        Ok(result) => {
            println!("Deleted {} documents", result.count);
            for doc_id in &result.doc_ids {
                println!("  Deleted document: {}", doc_id);
            }
        }
        Err(e) => eprintln!("Error deleting with filter: {}", e),
    }

    // 8. Delete a specific document by docID
    println!("\n=== Deleting Specific User Document ===");
    match get_document_ids(&client, base_url, "User").await {
        Ok(keys) => {
            if !keys.is_empty() {
                match delete_document(&client, base_url, "User", &keys[0]).await {
                    Ok(()) => println!("Successfully deleted document {}", keys[0]),
                    Err(e) => eprintln!("Error deleting document: {}", e),
                }
            } else {
                println!("No documents to delete");
            }
        }
        Err(e) => eprintln!("Error getting document keys: {}", e),
    }

    // 9. Working with different data types
    println!("\n=== Working with Complex Data Types ===");

    // First create a schema with various field types
    let complex_schema = r#"
        type Product {
            name: String
            price: Float
            inStock: Boolean
            tags: [String]
            metadata: JSON
            createdAt: DateTime
        }
    "#;

    let schema_url = format!("{}/schema", base_url);
    let _ = client
        .post(&schema_url)
        .header("Content-Type", "text/plain")
        .body(complex_schema)
        .send()
        .await;

    // Create a product with complex data
    let product = serde_json::json!({
        "name": "Laptop",
        "price": 999.99,
        "inStock": true,
        "tags": ["electronics", "computer", "portable"],
        "metadata": {
            "brand": "TechCorp",
            "model": "X1",
            "specifications": {
                "cpu": "Intel i7",
                "ram": "16GB",
                "storage": "512GB SSD"
            }
        },
        "createdAt": "2024-01-15T10:30:00Z"
    });

    match create_document(&client, base_url, "Product", &product).await {
        Ok(()) => println!("Created product"),
        Err(e) => eprintln!("Error creating product: {}", e),
    }

    // 10. Advanced filtering examples
    println!("\n=== Advanced Filtering Examples ===");

    // Create more users for filtering demos
    let demo_users = vec![
        serde_json::json!({"name": "John Doe", "email": "john@test.com", "age": 22}),
        serde_json::json!({"name": "Jane Doe", "email": "jane@test.com", "age": 33}),
        serde_json::json!({"name": "Mike Smith", "email": "mike@test.com", "age": 45}),
    ];

    let _ = create_documents(&client, base_url, "User", &demo_users).await;

    // Delete users with complex filter
    let complex_filter = serde_json::json!({
        "_or": [
            {"age": {"_lt": 25}},
            {"email": {"_like": "%test.com"}}
        ]
    });

    match delete_documents_with_filter(&client, base_url, "User", complex_filter).await {
        Ok(result) => {
            println!("Complex filter deleted {} documents", result.count);
        }
        Err(e) => eprintln!("Error with complex filter: {}", e),
    }

    println!("\n=== Collection Operations Tutorial Complete ===");
    println!("You've learned how to:");
    println!("- Create single and multiple documents");
    println!("- Retrieve documents by ID and get all document keys");
    println!("- Update documents individually and with filters");
    println!("- Delete documents individually and with filters");
    println!("- Work with complex data types (JSON, arrays, etc.)");
    println!("- Use advanced filtering with logical operators");

    Ok(())
}
