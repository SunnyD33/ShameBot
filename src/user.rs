use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const TROLL_THRESHOLD: i32 = 200; // Start pinging at 200 dollars
pub const SUPER_TROLL_THRESHOLD: i32 = 500; // Lay into the user at this point

// New structure: User has multiple games
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub user: String,
    pub games: HashMap<String, i32>, // game_name -> total
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Helper functions
fn load_user_file() -> Vec<User> {
    match std::fs::read_to_string("../users.json") {
        Ok(contents) => {
            if contents.is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&contents).unwrap_or_else(|_| Vec::new())
            }
        }
        Err(_) => Vec::new(),
    }
}

fn save_users_to_file(users: &Vec<User>) -> Result<()> {
    let json = serde_json::to_string_pretty(users)?;
    std::fs::write("../users.json", json)?;
    Ok(())
}

// Function to add a new game to an existing user
pub fn add_game(username: &str, game: &str, starting_total: &str) -> Result<()> {
    let mut users = load_user_file();
    let total: i32 = starting_total
        .parse()
        .map_err(|_| "Invalid number for starting total")?;

    // Find the user
    let user_found = users.iter_mut().find(|user| user.user == username);

    match user_found {
        Some(user) => {
            // User exists - check if game already exists
            if user.games.contains_key(game) {
                return Err(format!("User {} already has game '{}'", username, game).into());
            }

            // Add new game to existing user
            user.games.insert(game.to_string(), total);
            println!(
                "Added game '{}' with total {} to user '{}'",
                game, total, username
            );
        }
        None => {
            return Err(format!("User '{}' not found! Use !adduser first", username).into());
        }
    }

    save_users_to_file(&users)?;
    Ok(())
}

// Function to add a completely new user with their first game
pub fn add_user(username: &str, game: &str, starting_total: &str) -> Result<()> {
    let mut users = load_user_file();
    let total: i32 = starting_total
        .parse()
        .map_err(|_| "Invalid number for starting total")?;

    // Check if user already exists
    if users.iter().any(|user| user.user == username) {
        return Err(format!(
            "User '{}' already exists! Use !addgame to add more games",
            username
        )
        .into());
    }

    // Create new user with first game
    let mut games = HashMap::new();
    games.insert(game.to_string(), total);

    let new_user = User {
        user: username.to_string(),
        games,
    };

    users.push(new_user);
    save_users_to_file(&users)?;

    println!(
        "Added new user '{}' with game '{}' and total {}",
        username, game, total
    );
    Ok(())
}

// Updated function to update totals (now needs to specify which game)
pub fn update_total(username: &str, game: &str, additional_total: &str) -> Result<(i32, bool)> {
    let mut users = load_user_file();
    let additional: i32 = additional_total
        .parse()
        .map_err(|_| "Invalid number for additional total")?;

    // Find the user
    let user_found = users.iter_mut().find(|user| user.user == username);

    let mut new_total = 0;
    let mut crossed_threshold = false;

    match user_found {
        Some(user) => {
            // Check if user has this game
            if let Some(current_total) = user.games.get_mut(game) {
                let old_total = *current_total;
                *current_total += additional;
                new_total = *current_total;
                crossed_threshold = old_total < 300 && new_total >= 300;
                println!("Updated {}'s {} total to {}", username, game, new_total);
            } else {
                return Err(format!("User '{}' doesn't have game '{}'", username, game).into());
            }
        }
        None => {
            return Err(format!("User '{}' not found", username).into());
        }
    }

    save_users_to_file(&users)?;
    Ok((new_total, crossed_threshold))
}

// Function to get all users and their games (for listing)
pub fn get_users() -> Result<Vec<User>> {
    Ok(load_user_file())
}

// Function to get current total
pub fn get_game_total(username: &str, game: &str) -> Result<i32> {
    let users = load_user_file();

    match users.iter().find(|user| user.user == username) {
        Some(user) => match user.games.get(game) {
            Some(&total) => Ok(total),
            None => Err(format!("User '{}' doesn't have a game", username).into()),
        },
        None => Err(format!("User '{}' not found", username).into()),
    }
}

// Function to get total across ALL games for a user
pub fn get_user_total_all_games(username: &str) -> Result<i32> {
    let users = load_user_file();

    match users.iter().find(|user| user.user == username) {
        Some(user) => {
            let total: i32 = user.games.values().sum();
            Ok(total)
        }
        None => Err(format!("User '{}' not found", username).into()),
    }
}

// Function to get specific user's games
pub fn get_user_games(username: &str) -> Result<HashMap<String, i32>> {
    let users = load_user_file();

    match users.iter().find(|user| user.user == username) {
        Some(user) => Ok(user.games.clone()),
        None => Err(format!("User '{}' not found", username).into()),
    }
}

// Function to delete a game from a user
pub fn remove_game(username: &str, game: &str) -> Result<()> {
    let mut users = load_user_file();

    let user_found = users.iter_mut().find(|user| user.user == username);

    match user_found {
        Some(user) => {
            if user.games.remove(game).is_some() {
                println!("Removed game '{}' from user '{}'", game, username);

                // If user has no games left, optionally remove the user entirely
                if user.games.is_empty() {
                    users.retain(|u| u.user != username);
                    println!("User '{}' had no games left and was removed", username);
                }
            } else {
                return Err(format!("User '{}' doesn't have game '{}'", username, game).into());
            }
        }
        None => {
            return Err(format!("User '{}' not found", username).into());
        }
    }

    save_users_to_file(&users)?;
    Ok(())
}

// Function to delete an entire user (all their games)
pub fn delete_user(username: &str) -> Result<()> {
    let mut users = load_user_file();
    let original_len = users.len();

    // Remove the user entirely
    users.retain(|user| user.user != username);

    if users.len() < original_len {
        save_users_to_file(&users)?;
        println!("Deleted user '{}' and all their games", username);
        Ok(())
    } else {
        Err(format!("User '{}' not found", username).into())
    }
}
