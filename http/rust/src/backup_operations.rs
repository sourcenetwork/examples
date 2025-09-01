// DefraDB Backup Operations Tutorial
//
// This tutorial demonstrates how to export and import database backups in DefraDB.
// Backups allow you to create snapshots of your data for disaster recovery,
// migration between environments, or data archival purposes.

use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Deserialize)]
struct DefraError {
    error: String,
}

// Backup configuration structure
#[derive(Debug, Serialize)]
struct BackupConfig {
    // File path where the backup will be saved/loaded
    filepath: String,
    // Collections to include in the backup (empty array means all collections)
    collections: Vec<String>,
    // Format of the backup file (e.g., "json", "jsonl")
    format: String,
    // Whether to format the output JSON for readability
    pretty: bool,
}

// Export a database backup to a file
async fn export_backup(
    client: &reqwest::Client,
    base_url: &str,
    config: BackupConfig,
) -> Result<(), String> {
    let url = format!("{}/backup/export", base_url);

    let response = match client.post(&url).json(&config).send().await {
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

// Import a database backup from a file
async fn import_backup(
    client: &reqwest::Client,
    base_url: &str,
    config: BackupConfig,
) -> Result<(), String> {
    let url = format!("{}/backup/import", base_url);

    let response = match client.post(&url).json(&config).send().await {
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
async fn create_sample_data(client: &reqwest::Client, base_url: &str) -> Result<(), String> {
    // Create User schema
    let user_schema = r#"
        type User {
            name: String
            email: String
            age: Int
            department: String
        }
    "#;

    let schema_url = format!("{}/schema", base_url);
    let _ = client
        .post(&schema_url)
        .header("Content-Type", "text/plain")
        .body(user_schema)
        .send()
        .await;

    // Create sample users
    let users = vec![
        serde_json::json!({
            "name": "Alice Johnson",
            "email": "alice@company.com",
            "age": 28,
            "department": "Engineering"
        }),
        serde_json::json!({
            "name": "Bob Smith",
            "email": "bob@company.com",
            "age": 35,
            "department": "Marketing"
        }),
        serde_json::json!({
            "name": "Carol Davis",
            "email": "carol@company.com",
            "age": 42,
            "department": "HR"
        }),
    ];

    let collection_url = format!("{}/collections/User", base_url);
    for user in users {
        let _ = client.post(&collection_url).json(&user).send().await;
    }

    Ok(())
}

// Helper function to count documents in a collection
async fn count_documents(
    client: &reqwest::Client,
    base_url: &str,
    collection: &str,
) -> Result<usize, String> {
    let graphql_url = format!("{}/graphql", base_url);
    let query = format!(
        r#"
        query {{
            _count({}: {{}})
        }}
    "#,
        collection
    );

    let gql_request = serde_json::json!({
        "query": query
    });

    let response = match client.post(&graphql_url).json(&gql_request).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };

    if response.status() == 200 {
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(count) = result
            .get("data")
            .and_then(|d| d.get(&format!("_count({})", collection)))
        {
            Ok(count.as_u64().unwrap_or(0) as usize)
        } else {
            Ok(0)
        }
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = "http://localhost:9181/api/v0";
    let client = reqwest::Client::new();

    // 1. Create sample data for backup demonstration
    println!("=== 1. Creating Sample Data ===");
    match create_sample_data(&client, base_url).await {
        Ok(()) => println!("Sample data created successfully"),
        Err(e) => eprintln!("Error creating sample data: {}", e),
    }

    // Check how many documents we have
    match count_documents(&client, base_url, "User").await {
        Ok(count) => println!("Total users in database: {}", count),
        Err(e) => eprintln!("Error counting documents: {}", e),
    }

    // 2. Export full database backup
    println!("\n=== 2. Full Database Backup ===");
    let full_backup_config = BackupConfig {
        filepath: "/tmp/defradb_full_backup.json".to_string(),
        collections: vec![], // Empty array means all collections
        format: "json".to_string(),
        pretty: true,
    };

    match export_backup(&client, base_url, full_backup_config).await {
        Ok(()) => println!("Full database backup exported to /tmp/defradb_full_backup.json"),
        Err(e) => eprintln!("Error exporting full backup: {}", e),
    }

    // 3. Export specific collection backup
    println!("\n=== 3. Collection-Specific Backup ===");
    let user_backup_config = BackupConfig {
        filepath: "/tmp/defradb_users_backup.json".to_string(),
        collections: vec!["User".to_string()],
        format: "json".to_string(),
        pretty: true,
    };

    match export_backup(&client, base_url, user_backup_config).await {
        Ok(()) => println!("User collection backup exported to /tmp/defradb_users_backup.json"),
        Err(e) => eprintln!("Error exporting User backup: {}", e),
    }

    // 4. Export compact backup (JSONL format)
    println!("\n=== 4. Compact Backup (JSONL Format) ===");
    let compact_backup_config = BackupConfig {
        filepath: "/tmp/defradb_compact_backup.jsonl".to_string(),
        collections: vec!["User".to_string()],
        format: "jsonl".to_string(), // JSON Lines format - one JSON object per line
        pretty: false,               // Compact format for smaller file size
    };

    match export_backup(&client, base_url, compact_backup_config).await {
        Ok(()) => println!("Compact backup exported to /tmp/defradb_compact_backup.jsonl"),
        Err(e) => eprintln!("Error exporting compact backup: {}", e),
    }

    // 5. Multiple collection backup
    println!("\n=== 5. Multiple Collection Backup ===");

    // First create another collection for demonstration
    let product_schema = r#"
        type Product {
            name: String
            price: Float
            category: String
        }
    "#;

    let schema_url = format!("{}/schema", base_url);
    let _ = client
        .post(&schema_url)
        .header("Content-Type", "text/plain")
        .body(product_schema)
        .send()
        .await;

    // Add some product data
    let products = vec![
        serde_json::json!({
            "name": "Laptop",
            "price": 999.99,
            "category": "Electronics"
        }),
        serde_json::json!({
            "name": "Coffee Mug",
            "price": 15.99,
            "category": "Home"
        }),
    ];

    let product_collection_url = format!("{}/collections/Product", base_url);
    for product in products {
        let _ = client
            .post(&product_collection_url)
            .json(&product)
            .send()
            .await;
    }

    // Backup both User and Product collections
    let multi_collection_backup_config = BackupConfig {
        filepath: "/tmp/defradb_multi_collection_backup.json".to_string(),
        collections: vec!["User".to_string(), "Product".to_string()],
        format: "json".to_string(),
        pretty: true,
    };

    match export_backup(&client, base_url, multi_collection_backup_config).await {
        Ok(()) => println!(
            "Multi-collection backup exported to /tmp/defradb_multi_collection_backup.json"
        ),
        Err(e) => eprintln!("Error exporting multi-collection backup: {}", e),
    }

    // 6. Demonstrate backup restoration
    println!("\n=== 6. Backup Restoration Demo ===");
    println!("Note: Import operations will restore data from backup files.");
    println!("Uncomment the following code to test import functionality:");

    /*
    // Example import from previously created backup
    let import_config = BackupConfig {
        filepath: "/tmp/defradb_users_backup.json".to_string(),
        collections: vec!["User".to_string()],
        format: "json".to_string(),
        pretty: true,
    };

    match import_backup(&client, base_url, import_config).await {
        Ok(()) => println!("Successfully imported backup"),
        Err(e) => eprintln!("Error importing backup: {}", e),
    }
    */

    // 7. Backup best practices demonstration
    println!("\n=== 7. Backup Best Practices ===");

    // Timestamped backup filename
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let timestamped_backup_config = BackupConfig {
        filepath: format!("/tmp/defradb_backup_{}.json", timestamp),
        collections: vec![],
        format: "json".to_string(),
        pretty: false, // Use compact format for production backups
    };

    match export_backup(&client, base_url, timestamped_backup_config).await {
        Ok(()) => println!(
            "Timestamped backup exported to /tmp/defradb_backup_{}.json",
            timestamp
        ),
        Err(e) => eprintln!("Error exporting timestamped backup: {}", e),
    }

    // 8. Backup verification
    println!("\n=== 8. Backup Verification ===");
    println!("To verify backup integrity, you can:");
    println!("1. Check file exists and is not empty");
    println!("2. Parse JSON to ensure it's valid");
    println!("3. Import to a test database and verify data");

    // Example verification (check if backup file was created)
    use std::path::Path;
    let backup_path = "/tmp/defradb_users_backup.json";
    if Path::new(backup_path).exists() {
        println!("✓ Backup file exists at {}", backup_path);

        // Read and validate JSON structure
        match std::fs::read_to_string(backup_path) {
            Ok(contents) => match serde_json::from_str::<serde_json::Value>(&contents) {
                Ok(_) => println!("✓ Backup file contains valid JSON"),
                Err(e) => eprintln!("✗ Invalid JSON in backup file: {}", e),
            },
            Err(e) => eprintln!("✗ Error reading backup file: {}", e),
        }
    } else {
        println!("✗ Backup file not found at {}", backup_path);
    }

    println!("\n=== Backup Operations Tutorial Complete ===");
    println!("You've learned how to:");
    println!("- Export full database backups");
    println!("- Export collection-specific backups");
    println!("- Use different backup formats (JSON, JSONL)");
    println!("- Configure backup options (pretty printing, etc.)");
    println!("- Import backups (restoration process)");

    Ok(())
}
