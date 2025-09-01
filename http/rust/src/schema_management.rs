// DefraDB Schema Management Tutorial
//
// This tutorial demonstrates how to manage schemas in DefraDB using Rust.
// Schemas define the structure of your collections (similar to tables in SQL databases).
// DefraDB uses GraphQL Schema Definition Language (SDL) to define collection schemas.

use reqwest;
use serde::Deserialize;
use serde_json;

// Error response structure from DefraDB
#[derive(Debug, Deserialize)]
struct DefraError {
    error: String,
}

// Collection information returned when adding schemas
#[derive(Debug, Deserialize)]
struct Collection {
    #[serde(rename = "CollectionID")]
    collection_id: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "VersionID")]
    version_id: String,
    #[serde(rename = "Fields")]
    fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
struct Field {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Kind")]
    kind: u64,
    #[serde(rename = "FieldID")]
    field_id: String,
}

// Add a new schema to DefraDB
// Schemas define the structure of collections using GraphQL SDL
async fn add_schema(
    client: &reqwest::Client,
    base_url: &str,
    schema_sdl: String,
) -> Result<Vec<Collection>, String> {
    let schema_url = format!("{}/schema", base_url);

    let response = match client
        .post(&schema_url)
        .header("Content-Type", "text/plain")
        .body(schema_sdl)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        // println!("Response: {:?}", response.text().await);
        // Parse the response);
        let collections: Vec<Collection> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(collections)
    } else {
        let error: DefraError = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error: {}", e))?;
        Err(error.error)
    }
}

// Get collection information by name, schema ID, or version ID
async fn get_collections(
    client: &reqwest::Client,
    base_url: &str,
    name: Option<&str>,
    collection_id: Option<&str>,
    version_id: Option<&str>,
    get_inactive: bool,
) -> Result<Vec<Collection>, String> {
    let mut url = format!("{}/collections", base_url);
    let mut params = Vec::new();

    if let Some(name) = name {
        params.push(format!("name={}", name));
    }
    if let Some(id) = collection_id {
        params.push(format!("collection_id={}", id));
    }
    if let Some(id) = version_id {
        params.push(format!("version_id={}", id));
    }
    if get_inactive {
        params.push("get_inactive=true".to_string());
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        // Response can be a single collection or array of collections
        let text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        // Try to parse as array first, then as single collection
        if let Ok(collections) = serde_json::from_str::<Vec<Collection>>(&text) {
            Ok(collections)
        } else if let Ok(collection) = serde_json::from_str::<Collection>(&text) {
            Ok(vec![collection])
        } else {
            Err("Failed to parse collection response".to_string())
        }
    } else {
        let error: DefraError = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error: {}", e))?;
        Err(error.error)
    }
}

// Patch collection definitions using JSON Patch format
// Uses the PATCH /collections endpoint with JSON patch operations
async fn patch_collection(
    client: &reqwest::Client,
    base_url: &str,
    json_patch: serde_json::Value,
    migration: Option<serde_json::Value>,
) -> Result<(), String> {
    let url = format!("{}/collections", base_url);

    let patch_request = serde_json::json!({
        "Patch": json_patch.to_string(),
        "Migration": migration.unwrap_or(serde_json::json!({}))
    });

    let response = match client
        .patch(&url)
        .header("Content-Type", "application/json")
        .json(&patch_request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        Ok(())
    } else {
        let error: DefraError = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error: {}", e))?;
        Err(error.error)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = "http://localhost:9181/api/v0";
    let client = reqwest::Client::new();

    // 1. Add a simple schema for a User collection
    println!("=== Adding User Schema ===");
    let user_schema = r#"
        type User {
            name: String
            email: String
            age: Int
        }
    "#;

    match add_schema(&client, base_url, user_schema.to_string()).await {
        Ok(collections) => {
            println!("Successfully added User schema!");
            for collection in &collections {
                println!(
                    "Collection: {} (ID: {})",
                    collection.name, collection.collection_id
                );
                println!("Version: {}", collection.version_id);
                for field in &collection.fields {
                    println!("  Field: {} ({})", field.name, field.kind);
                }
            }
        }
        Err(e) => {
            if e.contains("already exists") {
                println!("User schema already exists, continuing...");
            } else {
                eprintln!("Error adding User schema: {}", e);
                return Ok(());
            }
        }
    }

    // 2. Add a more complex schema with relationships
    println!("\n=== Adding Blog Schema with Relationships ===");
    let blog_schema = r#"
        type Blog {
            title: String
            content: String
            author: User
            published: Boolean
            publishedAt: DateTime
            tags: [String]
        }
    "#;

    match add_schema(&client, base_url, blog_schema.to_string()).await {
        Ok(collections) => {
            println!("Successfully added Blog schema!");
            for collection in &collections {
                println!(
                    "Collection: {} (ID: {})",
                    collection.name, collection.collection_id
                );
            }
        }
        Err(e) => {
            if e.contains("already exists") {
                println!("Blog schema already exists, continuing...");
            } else {
                eprintln!("Error adding Blog schema: {}", e);
            }
        }
    }

    // 3. Get information about all collections
    println!("\n=== Listing All Collections ===");
    match get_collections(&client, base_url, None, None, None, false).await {
        Ok(collections) => {
            println!("Found {} collections:", collections.len());
            for collection in &collections {
                println!(
                    "  - {} (Version: {})",
                    collection.name, collection.version_id
                );
            }
        }
        Err(e) => eprintln!("Error listing collections: {}", e),
    }

    // 4. Get specific collection information
    println!("\n=== Getting User Collection Info ===");
    match get_collections(&client, base_url, Some("User"), None, None, false).await {
        Ok(collections) => {
            for collection in &collections {
                println!("Collection: {}", collection.name);
                println!("Collection ID: {}", collection.collection_id);
                println!("Version ID: {}", collection.version_id);
                println!("Fields:");
                for field in &collection.fields {
                    println!("  - {}: {}", field.name, field.kind);
                }
            }
        }
        Err(e) => eprintln!("Error getting User collection: {}", e),
    }

    // 5. Schema versioning example using JSON Patch
    println!("\n=== Schema Versioning with JSON Patch ===");
    // Adding a new field to an existing schema creates a new version
    let version_patch = serde_json::json!([
        {
            "op": "add",
            "path": "/User/Fields/-",
            "value": {
                "Name": "profile_picture",
                "Kind": "String"
            }
        }
    ]);

    match patch_collection(&client, base_url, version_patch, None).await {
        Ok(()) => {
            println!("Successfully created User collectio v2 via JSON patch!");

            // Get the updated collection to see the new version
            match get_collections(&client, base_url, Some("User"), None, None, false).await {
                Ok(collections) => {
                    for collection in &collections {
                        println!("New version ID: {}", collection.version_id);
                        println!("Fields in new version:");
                        for field in &collection.fields {
                            println!("  - {}: kind={}", field.name, field.kind);
                        }

                        // The patched version automatically becomes the default version
                        println!("This version is now the default version for User collection");
                    }
                }
                Err(e) => eprintln!("Error getting updated collection: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Error creating User schema v2: {}", e);
        }
    }

    // 6. Additional JSON Patch operations example
    println!("\n=== Additional JSON Patch Operations ===");

    // Add multiple fields to the User collection using JSON Patch
    let additional_patch = serde_json::json!([
        {
            "op": "add",
            "path": "/User/Fields/-",
            "value": {
                "Name": "bio",
                "Kind": 11  // String type
            }
        },
        {
            "op": "add",
            "path": "/User/Fields/-",
            "value": {
                "Name": "is_verified",
                "Kind": 4   // Boolean type
            }
        }
    ]);

    match patch_collection(&client, base_url, additional_patch, None).await {
        Ok(()) => {
            println!("Successfully applied additional JSON patch to User collection!");

            // Get the updated collection information to verify the patch
            match get_collections(&client, base_url, Some("User"), None, None, false).await {
                Ok(collections) => {
                    for collection in &collections {
                        println!(
                            "Updated collection: {} (Version: {})",
                            collection.name, collection.version_id
                        );
                        println!("All fields after additional patch:");
                        for field in &collection.fields {
                            println!(
                                "  - {}: kind={} (ID: {})",
                                field.name, field.kind, field.field_id
                            );
                        }
                    }
                }
                Err(e) => eprintln!("Error getting updated collection info: {}", e),
            }
        }
        Err(e) => {
            if e.contains("no changes") || e.contains("already exists") {
                println!("Additional patch already applied or no changes needed");
            } else {
                eprintln!("Error applying additional JSON patch: {}", e);
                println!("This might be expected if the fields already exist.");
            }
        }
    }

    // 7. JSON Patch operations reference
    println!("\n=== JSON Patch Operations Examples ===");

    println!("Common JSON Patch operations for collection schemas:");

    // Add field example
    let add_field_patch = serde_json::json!([
        {
            "op": "add",
            "path": "/CollectionName/Fields/-",
            "value": {
                "Name": "new_field",
                "Kind": 11  // String
            }
        }
    ]);
    println!("Add field patch:");
    println!("{}", serde_json::to_string_pretty(&add_field_patch)?);

    // Remove field example (note: field removal might have restrictions)
    let remove_field_patch = serde_json::json!([
        {
            "op": "remove",
            "path": "/CollectionName/Fields/2"  // Remove field at index 2
        }
    ]);
    println!("\nRemove field patch:");
    println!("{}", serde_json::to_string_pretty(&remove_field_patch)?);

    // Replace field kind example
    let replace_field_patch = serde_json::json!([
        {
            "op": "replace",
            "path": "/CollectionName/Fields/1/Kind",
            "value": 4  // Change to Boolean
        }
    ]);
    println!("\nReplace field kind patch:");
    println!("{}", serde_json::to_string_pretty(&replace_field_patch)?);

    println!("\nField Kind Reference:");
    println!("  1 = Bool (Boolean)");
    println!("  2 = Int (Integer)");
    println!("  3 = Float");
    println!("  4 = Boolean");
    println!("  11 = String");
    println!("  12 = Blob (Binary data)");
    println!("  13 = DateTime");
    println!("  14 = JSON");
    // Note: These kind values are based on common GraphQL scalar types

    Ok(())
}
