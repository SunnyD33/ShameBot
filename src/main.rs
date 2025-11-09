use dotenv::dotenv;
use std::env;

use serenity::async_trait;
use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod user;

struct Handler;

// Helper function to parse commands with quotes (like Unix shell)
fn parse_command_with_quotes(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current_part = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                // Don't include the quote character in the result
            }
            ' ' if !in_quotes => {
                if !current_part.is_empty() {
                    parts.push(current_part.clone());
                    current_part.clear();
                }
            }
            _ => {
                current_part.push(ch);
            }
        }
    }

    // Don't forget the last part
    if !current_part.is_empty() {
        parts.push(current_part);
    }

    parts
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Help message
        if msg.content == "!help" || msg.content == "!commands" {
            let help_embed = CreateEmbed::new()
                .title("🎮 ShameBot - Command List")
                .description("Track your gaming totals across different games! When the totals get high, it puts you on blast for your spending!")
                .color(0x00ff00) // Green color
                .field(
                            "👤 User Management",
                            "• `!adduser <user> \"<game>\" <total>` - Create new user with first game\n• `!deleteuser <user>` - Delete user and all their games",
                            false
                        )
                        .field(
                            "🎯 Game Management",
                            "• `!addgame <user> \"<game>\" <total>` - Add new game to existing user\n• `!removegame <user> \"<game>\"` - Remove specific game from user\n• `!updatetotal <user> \"<game>\" <amount>` - Add money to game total",
                            false
                        )
                        .field(
                            "📊 Information & Viewing",
                            "• `!getusers` - Show all users and their games\n• `!usergames <user>` - Show all games for specific user\n• `!gametotal <user> \"<game>\"` - Show total for specific game\n• `!usertotal <user>` - Show user's total across all games\n• `!help` or `!commands` - Show this help message",
                            false
                        )
                        .field(
                            "💡 Command Examples",
                            "```\n!adduser Q \"Tekken 8\" 200\n!addgame Alice \"Street Fighter 6\" 150\n!updatetotal Q \"Tekken 8\" 50\n!usergames Q\n!gametotal Q \"Tekken 8\"\n!usertotal Q\n!removegame Alice \"Street Fighter 6\"\n!deleteuser Bob```",
                            false
                        )
                        .field(
                            "⚠️ Important Notes",
                            "• Use quotes around game names with spaces\n• Game names are case-sensitive\n• Amounts must be valid numbers\n• User names cannot contain spaces",
                            false
                        );

            let builder = CreateMessage::new().embed(help_embed);

            if let Err(error) = msg.channel_id.send_message(&ctx.http, builder).await {
                println!("Error sending help message: {error:?}");
                // Fallback to simple text if embed fails
                let fallback_text = r#"
        **🎮 Game Tracker Bot Commands:**

        **User Management:**
        • !adduser <user> "<game>" <total> - Create new user
        • !deleteuser <user> - Delete user and all games

        **Game Management:**
        • !addgame <user> "<game>" <total> - Add game to user
        • !removegame <user> "<game>" - Remove game from user
        • !updatetotal <user> "<game>" <amount> - Add to game total

        **Information:**
        • !getusers - Show all users and games
        • !usergames <user> - Show user's games
        • !help - Show commands

        **Examples:**
        !adduser Q "Tekken 8" 200
        !updatetotal Q "Tekken 8" 50

        **Note:** Use quotes around game names with spaces!
        "#;
                msg.channel_id.say(&ctx.http, fallback_text).await.ok();
            }
        }

        if msg.content == "!quickhelp" {
            let quick_help = "**Quick Commands:** `!adduser`, `!addgame`, `!updatetotal`, `!getusers`, `!usergames`, `!deleteuser`, `!removegame` | Use `!help` for details";
            msg.channel_id.say(&ctx.http, quick_help).await.ok();
        }

        // !adduser Q "Tekken 8" 200
        if msg.content.starts_with("!adduser") {
            let parts = parse_command_with_quotes(&msg.content);

            if parts.len() != 4 {
                msg.channel_id
                    .say(
                        &ctx.http,
                        "Usage: !adduser <username> \"<game name>\" <total>",
                    )
                    .await
                    .ok();
                return;
            }

            let username = &parts[1];
            let game = &parts[2];
            let total = &parts[3];

            match user::add_user(username, game, total) {
                Ok(_) => {
                    let mes = format!(
                        "Added user {} with game '{}' and total ${}",
                        username, game, total
                    );
                    msg.channel_id.say(&ctx.http, mes).await.ok();
                }
                Err(e) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Error: {}", e))
                        .await
                        .ok();
                }
            }
        }

        // !addgame Q "Street Fighter 6" 150
        if msg.content.starts_with("!addgame") {
            let parts = parse_command_with_quotes(&msg.content);

            if parts.len() != 4 {
                msg.channel_id
                    .say(
                        &ctx.http,
                        "Usage: !addgame <username> \"<game name>\" <starting_total>",
                    )
                    .await
                    .ok();
                return;
            }

            let username = &parts[1];
            let game = &parts[2];
            let total = &parts[3];

            match user::add_game(username, game, total) {
                Ok(_) => {
                    let mes = format!(
                        "Added game '{}' with total ${} to user {}",
                        game, total, username
                    );
                    msg.channel_id.say(&ctx.http, mes).await.ok();
                }
                Err(e) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Error: {}", e))
                        .await
                        .ok();
                }
            }
        }

        // !updatetotal Q "Tekken 8" 50
        if msg.content.starts_with("!updatetotal") {
            let parts = parse_command_with_quotes(&msg.content);

            if parts.len() != 4 {
                msg.channel_id
                    .say(
                        &ctx.http,
                        "Usage: !updatetotal <username> \"<game name>\" <additional_amount>",
                    )
                    .await
                    .ok();
                return;
            }

            let username = &parts[1];
            let game = &parts[2];
            let total = &parts[3];

            match user::update_total(username, game, total) {
                Ok((new_total, crossed_threshold)) => {
                    let mes = format!(
                        "{}'s total for '{}' was updated by ${}",
                        username, game, new_total
                    );
                    msg.channel_id.say(&ctx.http, mes).await.ok();

                    if crossed_threshold {
                        let troll_msg =
                            format!("@here 🚨 {} just crossed $300 in {}! 💸", username, game);
                        msg.channel_id.say(&ctx.http, troll_msg).await.ok();
                    }
                }
                Err(e) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Error: {}", e))
                        .await
                        .ok();
                }
            }
        }

        // !removegame Q "Tekken 8"
        if msg.content.starts_with("!removegame") {
            let parts = parse_command_with_quotes(&msg.content);

            if parts.len() != 3 {
                msg.channel_id
                    .say(&ctx.http, "Usage: !removegame <username> \"<game name>\"")
                    .await
                    .ok();
                return;
            }

            let username = &parts[1];
            let game = &parts[2];

            match user::remove_game(username, game) {
                Ok(_) => {
                    let mes = format!("Removed game '{}' from user {}", game, username);
                    msg.channel_id.say(&ctx.http, mes).await.ok();
                }
                Err(e) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Error: {}", e))
                        .await
                        .ok();
                }
            }
        }

        // !deleteuser Q
        if msg.content.starts_with("!deleteuser") {
            let parts = parse_command_with_quotes(&msg.content);

            if parts.len() != 2 {
                msg.channel_id
                    .say(&ctx.http, "Usage: !deleteuser <username>")
                    .await
                    .ok();
                return;
            }

            let username = &parts[1];

            match user::delete_user(username) {
                Ok(_) => {
                    let mes = format!("Deleted user {} and all their games", username);
                    msg.channel_id.say(&ctx.http, mes).await.ok();
                }
                Err(e) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Error: {}", e))
                        .await
                        .ok();
                }
            }
        }

        // !usergames Q - show all games for a specific user
        if msg.content.starts_with("!usergames") {
            let parts = parse_command_with_quotes(&msg.content);

            if parts.len() != 2 {
                msg.channel_id
                    .say(&ctx.http, "Usage: !usergames <username>")
                    .await
                    .ok();
                return;
            }

            let username = &parts[1];

            match user::get_user_games(username) {
                Ok(games) => {
                    if games.is_empty() {
                        msg.channel_id
                            .say(&ctx.http, format!("User {} has no games", username))
                            .await
                            .ok();
                    } else {
                        let games_list: Vec<String> = games
                            .iter()
                            .map(|(game, total)| format!("• {}: ${}", game, total))
                            .collect();

                        let mes = format!("**{}'s Games:**\n{}", username, games_list.join("\n"));
                        msg.channel_id.say(&ctx.http, mes).await.ok();
                    }
                }
                Err(e) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Error: {}", e))
                        .await
                        .ok();
                }
            }
        }

        // !getusers - show all users (updated for new structure)
        if msg.content == "!getusers" {
            match user::get_users() {
                Ok(user_list) => {
                    if user_list.is_empty() {
                        msg.channel_id.say(&ctx.http, "No users are currently added to the bot! Try the !adduser command.").await.ok();
                        return;
                    }

                    let user_strings: Vec<String> = user_list
                        .iter()
                        .map(|user| {
                            let games_info: Vec<String> = user
                                .games
                                .iter()
                                .map(|(game, total)| format!("  • {}: ${}", game, total))
                                .collect();

                            format!("**{}**\n{}", user.user, games_info.join("\n"))
                        })
                        .collect();

                    let mes = format!("**All Users:**\n{}", user_strings.join("\n\n"));
                    msg.channel_id.say(&ctx.http, mes).await.ok();
                }
                Err(e) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Error: {}", e))
                        .await
                        .ok();
                }
            }
        }

        if msg.content.starts_with("!gametotal") {
            let parts = parse_command_with_quotes(&msg.content);

            if parts.len() != 3 {
                msg.channel_id
                    .say(&ctx.http, "Usage: !gametotal <username> \"<game>\"")
                    .await
                    .ok();
                return;
            }

            let username = &parts[1];
            let game = &parts[2];

            match user::get_game_total(username, game) {
                Ok(total) => {
                    let mes = format!("{}'s total for '{}': ${}", username, game, total);
                    msg.channel_id.say(&ctx.http, mes).await.ok();
                }
                Err(e) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Error: {}", e))
                        .await
                        .ok();
                }
            }
        }

        if msg.content.starts_with("!usertotal") {
            let parts = parse_command_with_quotes(&msg.content);

            if parts.len() != 2 {
                msg.channel_id
                    .say(&ctx.http, "Usage: !usertotal <username>")
                    .await
                    .ok();
                return;
            }

            let username = &parts[1];

            match user::get_user_total_all_games(username) {
                Ok(total) => {
                    let mes = format!(
                        "{}'s total across all available games: ${}",
                        username, total
                    );
                    msg.channel_id.say(&ctx.http, mes).await.ok();
                }
                Err(e) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Error: {}", e))
                        .await
                        .ok();
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    //Load environment variables
    dotenv().ok();

    println!("🚀 Starting ShameBot...");

    // Load token environment variable
    let token = env::var("DISCORD_TOKEN").expect("No token was found in the environment");

    // Set Intents for the bot
    let intents = GatewayIntents::all();

    //Create instance of the client, logging in the bot
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("There was an issue creating the client. Check bot setup");

    // Listen for commands after client is started and bot is logged in
    if let Err(error) = client.start().await {
        println!("Client error: {error:?}");
    }
}
