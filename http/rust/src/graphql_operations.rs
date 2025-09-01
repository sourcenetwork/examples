// DefraDB GraphQL Operations Tutorial
//
// This tutorial demonstrates how to interact with DefraDB using GraphQL.
// GraphQL is the primary query language for DefraDB and provides a flexible
// way to query and mutate your data. DefraDB automatically generates a GraphQL
// schema based on your collection schemas.

use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Deserialize)]
struct DefraError {
    error: String,
}

// GraphQL request structure
#[derive(Debug, Serialize)]
struct GraphQLRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation_name: Option<String>,
}

// GraphQL response structure
#[derive(Debug, Deserialize)]
struct GraphQLResponse {
    data: Option<serde_json::Value>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

// Sample data structures for examples
#[derive(Debug, Serialize, Deserialize)]
struct User {
    #[serde(rename = "_docID", skip_serializing_if = "Option::is_none")]
    doc_id: Option<String>,
    name: String,
    email: String,
    age: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Blog {
    #[serde(rename = "_docID", skip_serializing_if = "Option::is_none")]
    doc_id: Option<String>,
    title: String,
    content: String,
    published: bool,
    author_id: Option<String>,
}

// Execute a GraphQL query or mutation
async fn execute_graphql(
    client: &reqwest::Client, 
    base_url: &str, 
    request: GraphQLRequest
) -> Result<GraphQLResponse, String> {
    let url = format!("{}/graphql", base_url);
    
    let response = match client
        .post(&url)
        .json(&request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };
    
    if response.status() == 200 {
        let gql_response: GraphQLResponse = response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(gql_response)
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

// Execute a GraphQL query using GET (useful for simple queries)
async fn execute_graphql_get(
    client: &reqwest::Client, 
    base_url: &str, 
    query: &str
) -> Result<GraphQLResponse, String> {
    let url = format!("{}/graphql?query={}", base_url, urlencoding::encode(query));
    
    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(e) => return Err(format!("Request failed: {}", e)),
    };
    
    if response.status() == 200 {
        let gql_response: GraphQLResponse = response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(gql_response)
    } else {
        let error: DefraError = response.json().await.unwrap();
        Err(error.error)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = "http://localhost:9181/api/v0";
    let client = reqwest::Client::new();
    
    // Setup schemas for our examples
    println!("=== Setting up Schemas ===");
    let schemas = vec![
        r#"type User {
            name: String
            email: String  
            age: Int
        }"#,
        r#"type Blog {
            title: String
            content: String
            published: Boolean
            author: User
        }"#
    ];
    
    for schema in schemas {
        let schema_url = format!("{}/schema", base_url);
        let _ = client
            .post(&schema_url)
            .header("Content-Type", "text/plain")
            .body(schema)
            .send()
            .await;
    }
    
    // 1. Simple Query - Get all users
    println!("\n=== 1. Simple Query: Get All Users ===");
    let query_all_users = GraphQLRequest {
        query: r#"
            query {
                User {
                    _docID
                    name
                    email
                    age
                }
            }
        "#.to_string(),
        variables: None,
        operation_name: None,
    };
    
    match execute_graphql(&client, base_url, query_all_users).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("Users found:");
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
            if let Some(errors) = response.errors {
                for error in errors {
                    eprintln!("GraphQL Error: {}", error.message);
                }
            }
        },
        Err(e) => eprintln!("Error executing query: {}", e),
    }
    
    // 2. Query with Variables and Filters
    println!("\n=== 2. Query with Variables and Filters ===");
    let query_with_filter = GraphQLRequest {
        query: r#"
            query GetUsersByAge($minAge: Int!) {
                User(filter: {age: {_gt: $minAge}}) {
                    _docID
                    name
                    age
                }
            }
        "#.to_string(),
        variables: Some(serde_json::json!({
            "minAge": 25
        })),
        operation_name: Some("GetUsersByAge".to_string()),
    };
    
    match execute_graphql(&client, base_url, query_with_filter).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("Users older than 25:");
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        },
        Err(e) => eprintln!("Error executing filtered query: {}", e),
    }
    
    // 3. Create Mutation - Add a new user
    println!("\n=== 3. Create Mutation ===");
    let create_user = GraphQLRequest {
        query: r#"
            mutation CreateUser($input: [UserMutationInputArg!]!) {
                create_User(input: $input) {
                    _docID
                    name
                    email
                    age
                }
            }
        "#.to_string(),
        variables: Some(serde_json::json!({
            "input": [{
                "name": "GraphQL User",
                "email": "graphql@example.com", 
                "age": 27
            }]
        })),
        operation_name: Some("CreateUser".to_string()),
    };
    
    match execute_graphql(&client, base_url, create_user).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("Created user:");
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
            if let Some(errors) = response.errors {
                for error in errors {
                    eprintln!("GraphQL Error: {}", error.message);
                }
            }
        },
        Err(e) => eprintln!("Error creating user: {}", e),
    }
    
    // 4. Update Mutation - Update user data
    println!("\n=== 4. Update Mutation ===");
    let update_user = GraphQLRequest {
        query: r#"
            mutation UpdateUsers($filter: UserFilterArg, $input: UserMutationInputArg!) {
                update_User(filter: $filter, input: $input) {
                    _docID
                    name
                    email
                    age
                }
            }
        "#.to_string(),
        variables: Some(serde_json::json!({
            "filter": {
                "name": {"_eq": "GraphQL User"}
            },
            "input": {
                "age": 28
            }
        })),
        operation_name: Some("UpdateUsers".to_string()),
    };
    
    match execute_graphql(&client, base_url, update_user).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("Updated user:");
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        },
        Err(e) => eprintln!("Error updating user: {}", e),
    }
    
    // 5. Delete Mutation
    println!("\n=== 5. Delete Mutation ===");
    let delete_user = GraphQLRequest {
        query: r#"
            mutation DeleteUsers($filter: UserFilterArg) {
                delete_User(filter: $filter) {
                    _docID
                }
            }
        "#.to_string(),
        variables: Some(serde_json::json!({
            "filter": {
                "email": {"_eq": "graphql@example.com"}
            }
        })),
        operation_name: Some("DeleteUsers".to_string()),
    };
    
    match execute_graphql(&client, base_url, delete_user).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("Deleted user:");
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        },
        Err(e) => eprintln!("Error deleting user: {}", e),
    }
    
    // 6. Relationship Queries
    println!("\n=== 6. Relationship Queries ===");
    
    // First create a user and a blog
    let create_author = GraphQLRequest {
        query: r#"
            mutation {
                create_User(input: [{
                    name: "Blog Author",
                    email: "author@blog.com",
                    age: 35
                }]) {
                    _docID
                    name
                }
            }
        "#.to_string(),
        variables: None,
        operation_name: None,
    };
    
    let author_id = match execute_graphql(&client, base_url, create_author).await {
        Ok(response) => {
            if let Some(data) = response.data {
                // Extract the docID from the response
                data["create_User"][0]["_docID"].as_str().unwrap_or("").to_string()
            } else {
                String::new()
            }
        },
        Err(e) => {
            eprintln!("Error creating author: {}", e);
            String::new()
        }
    };
    
    if !author_id.is_empty() {
        // Create a blog post linked to this author
        let create_blog = GraphQLRequest {
            query: r#"
                mutation CreateBlog($input: [BlogMutationInputArg!]!) {
                    create_Blog(input: $input) {
                        _docID
                        title
                        content
                        author {
                            _docID
                            name
                            email
                        }
                    }
                }
            "#.to_string(),
            variables: Some(serde_json::json!({
                "input": [{
                    "title": "My First Blog Post",
                    "content": "This is the content of my first blog post written using GraphQL!",
                    "published": true,
                    "author_id": author_id
                }]
            })),
            operation_name: Some("CreateBlog".to_string()),
        };
        
        match execute_graphql(&client, base_url, create_blog).await {
            Ok(response) => {
                if let Some(data) = response.data {
                    println!("Created blog with author relationship:");
                    println!("{}", serde_json::to_string_pretty(&data)?);
                }
            },
            Err(e) => eprintln!("Error creating blog: {}", e),
        }
    }
    
    // 7. Advanced Queries with Aggregation
    println!("\n=== 7. Advanced Queries with Aggregation ===");
    let aggregation_query = GraphQLRequest {
        query: r#"
            query {
                User {
                    name
                    age
                }
                _count(User: {})
                _avg(User: {field: age})
                _max(User: {field: age})
                _min(User: {field: age})
            }
        "#.to_string(),
        variables: None,
        operation_name: None,
    };
    
    match execute_graphql(&client, base_url, aggregation_query).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("User statistics:");
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        },
        Err(e) => eprintln!("Error executing aggregation query: {}", e),
    }
    
    // 8. Using GraphQL GET endpoint for simple queries
    println!("\n=== 8. GraphQL GET Endpoint ===");
    let simple_query = "{ User { name email } }";
    
    match execute_graphql_get(&client, base_url, simple_query).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("Users via GET:");
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        },
        Err(e) => eprintln!("Error with GET query: {}", e),
    }
    
    // 9. Sorting and Pagination
    println!("\n=== 9. Sorting and Pagination ===");
    let sorted_query = GraphQLRequest {
        query: r#"
            query {
                User(
                    order: {age: DESC}
                    limit: 5
                    offset: 0
                ) {
                    _docID
                    name
                    age
                }
            }
        "#.to_string(),
        variables: None,
        operation_name: None,
    };
    
    match execute_graphql(&client, base_url, sorted_query).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("Users sorted by age (descending), limited to 5:");
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        },
        Err(e) => eprintln!("Error with sorted query: {}", e),
    }
    
    // 10. Complex Filtering
    println!("\n=== 10. Complex Filtering ===");
    let complex_filter_query = GraphQLRequest {
        query: r#"
            query ComplexFilter($ageRange: IntOperatorBlockArg!, $emailPattern: StringOperatorBlockArg!) {
                User(filter: {
                    _and: [
                        {age: $ageRange},
                        {email: $emailPattern}
                    ]
                }) {
                    _docID
                    name
                    email
                    age
                }
            }
        "#.to_string(),
        variables: Some(serde_json::json!({
            "ageRange": {"_gte": 25, "_lte": 40},
            "emailPattern": {"_like": "%example.com"}
        })),
        operation_name: Some("ComplexFilter".to_string()),
    };
    
    match execute_graphql(&client, base_url, complex_filter_query).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("Users with complex filter (age 25-40 and email containing 'example.com'):");
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        },
        Err(e) => eprintln!("Error with complex filter: {}", e),
    }
    
    // 11. Introspection - Discover the schema
    println!("\n=== 11. Schema Introspection ===");
    let introspection_query = GraphQLRequest {
        query: r#"
            query IntrospectionQuery {
                __schema {
                    types {
                        name
                        kind
                        fields {
                            name
                            type {
                                name
                                kind
                            }
                        }
                    }
                }
            }
        "#.to_string(),
        variables: None,
        operation_name: Some("IntrospectionQuery".to_string()),
    };
    
    match execute_graphql(&client, base_url, introspection_query).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("Schema information:");
                // Filter to show only our custom types (not built-in GraphQL types)
                if let Some(schema) = data.get("__schema") {
                    if let Some(types) = schema.get("types") {
                        if let Some(types_array) = types.as_array() {
                            for type_def in types_array {
                                if let Some(name) = type_def.get("name").and_then(|n| n.as_str()) {
                                    if !name.starts_with("__") && !name.starts_with("String") && 
                                       !name.starts_with("Int") && !name.starts_with("Boolean") &&
                                       (name == "User" || name == "Blog") {
                                        println!("Type: {}", name);
                                        if let Some(fields) = type_def.get("fields") {
                                            if let Some(fields_array) = fields.as_array() {
                                                for field in fields_array {
                                                    if let Some(field_name) = field.get("name").and_then(|n| n.as_str()) {
                                                        println!("  Field: {}", field_name);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        Err(e) => eprintln!("Error with introspection: {}", e),
    }
    
    // 12. Batch Operations
    println!("\n=== 12. Batch Operations ===");
    let batch_create = GraphQLRequest {
        query: r#"
            mutation BatchCreate($users: [UserMutationInputArg!]!) {
                create_User(input: $users) {
                    _docID
                    name
                    email
                }
            }
        "#.to_string(),
        variables: Some(serde_json::json!({
            "users": [
                {"name": "Batch User 1", "email": "batch1@example.com", "age": 30},
                {"name": "Batch User 2", "email": "batch2@example.com", "age": 32},
                {"name": "Batch User 3", "email": "batch3@example.com", "age": 34}
            ]
        })),
        operation_name: Some("BatchCreate".to_string()),
    };
    
    match execute_graphql(&client, base_url, batch_create).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("Batch created users:");
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        },
        Err(e) => eprintln!("Error with batch create: {}", e),
    }
    
    // 13. Subscription-like Queries (Document History)
    println!("\n=== 13. Document History and Commits ===");
    let history_query = GraphQLRequest {
        query: r#"
            query {
                commits {
                    cid
                    collectionID
                    delta
                    docID
                    fieldId
                    fieldName
                    height
                    links {
                        cid
                        name
                    }
                }
            }
        "#.to_string(),
        variables: None,
        operation_name: None,
    };
    
    match execute_graphql(&client, base_url, history_query).await {
        Ok(response) => {
            if let Some(data) = response.data {
                println!("Recent commits:");
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        },
        Err(e) => eprintln!("Error querying commits: {}", e),
    }
    
    println!("\n=== GraphQL Operations Tutorial Complete ===");
    println!("You've learned how to:");
    println!("- Execute basic queries and mutations");
    println!("- Use variables and operation names");
    println!("- Apply filters, sorting, and pagination");
    println!("- Work with relationships between collections");
    println!("- Use GraphQL introspection to discover schema");
    println!("- Perform batch operations");
    println!("- Query document history and commits");
    println!("- Use both POST and GET GraphQL endpoints");
    
    Ok(())
}