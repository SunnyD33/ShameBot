use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub user: String,
    pub total: i32,
}

pub async fn test_database_connection() -> Result<()> {
    println!("🔍 Testing database connection...");

    // Create in-memory database for testing
    let conn = Connection::open_in_memory()?;

    // Create users table
    conn.execute(
        "CREATE TABLE users (
        id   INTEGER PRIMARY KEY,
        user TEXT NOT NULL,
        total INTEGER)",
        (),
    )?;
    println!("✅ Table created successfully");

    // Insert test data
    let test_user = User {
        id: 0,
        user: "TestUser".to_string(),
        total: 100,
    };

    conn.execute(
        "INSERT INTO users (user, total) VALUES (?1, ?2)",
        (&test_user.user, &test_user.total),
    )?;
    println!("✅ Test data inserted successfully");

    // Query and verify data
    let mut stmt = conn.prepare("SELECT id, user, total FROM users")?;
    let user_iter = stmt.query_map([], |row| {
        Ok(User {
            id: row.get(0)?,
            user: row.get(1)?,
            total: row.get(2)?,
        })
    })?;

    println!("📊 Retrieved data:");
    for user in user_iter {
        match user {
            Ok(u) => println!("   User: {:?}", u),
            Err(e) => println!("   Error retrieving user: {}", e),
        }
    }

    // Test additional operations
    test_database_operations(&conn)?;

    println!("✅ Database connection test completed successfully!");
    Ok(())
}

fn test_database_operations(conn: &Connection) -> Result<()> {
    println!("🧪 Testing additional database operations...");

    // Test update operation
    conn.execute(
        "UPDATE users SET total = ?1 WHERE user = ?2",
        (150, "TestUser"),
    )?;
    println!("✅ Update operation successful");

    // Test count query
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    println!("✅ Count query successful: {} users in database", count);

    // Test delete operation (optional)
    conn.execute("DELETE FROM users WHERE user = ?1", ["TestUser"])?;
    println!("✅ Delete operation successful");

    Ok(())
}
