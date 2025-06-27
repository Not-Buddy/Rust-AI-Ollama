// Import required dependencies
use clap::Parser;
use std::io::{self, Write};

// Import our custom module
mod connecttoollama;

#[derive(Parser)]
#[command(name = "Ollama Client")]
#[command(about = "A client for interacting with Ollama servers")]
struct Args {
    /// Skip the menu and run with a direct prompt
    #[arg(short, long)]
    prompt: Option<String>,
    
    /// Skip the menu and test connection
    #[arg(short, long)]
    test: bool,
}

fn display_menu() {
    println!("\n=== Ollama Client Menu ===");
    println!("1. Generate Response (Interactive)");
    println!("2. Test Server Connection");
    println!("3. View Configuration");
    println!("4. Exit");
    print!("Choose an option (1-4): ");
    io::stdout().flush().unwrap();
}

fn get_user_choice() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

fn display_config() {
    dotenv::dotenv().ok();
    
    println!("\n=== Current Configuration ===");
    
    match std::env::var("server_ip") {
        Ok(ip) => println!("Server IP: {}", ip),
        Err(_) => println!("Server IP: Not set in .env file"),
    }
    
    match std::env::var("model") {
        Ok(model) => println!("Model: {}", model),
        Err(_) => println!("Model: llama3.2 (default)"),
    }
    
    println!("================================");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Handle command line arguments
    if args.test {
        connecttoollama::test_connection().await?;
        return Ok(());
    }
    
    if let Some(prompt) = args.prompt {
        connecttoollama::generate_with_prompt(prompt).await?;
        return Ok(());
    }
    
    // Interactive menu
    loop {
        display_menu();
        
        match get_user_choice().as_str() {
            "1" => {
                match connecttoollama::generate_response().await {
                    Ok(_) => println!("✅ Generation completed successfully!"),
                    Err(e) => println!("❌ Error: {}", e),
                }
            },
            "2" => {
                match connecttoollama::test_connection().await {
                    Ok(_) => {},
                    Err(e) => println!("❌ Error: {}", e),
                }
            },
            "3" => {
                display_config();
            },
            "4" => {
                println!("👋 Goodbye!");
                break;
            },
            _ => {
                println!("❌ Invalid option. Please choose 1-4.");
            }
        }
        
        // Add a pause before showing menu again
        println!("\nPress Enter to continue...");
        let mut _input = String::new();
        io::stdin().read_line(&mut _input).unwrap();
    }
    
    Ok(())
}
