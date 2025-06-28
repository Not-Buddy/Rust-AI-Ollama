// Import required dependencies
use clap::Parser;
use std::io::{self, Write};

// Import our custom modules
mod connecttoollama;
mod connectlocally;
mod imagedescriber;  // Add this new import

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
    
    /// Use local Ollama instance instead of remote server
    #[arg(short, long)]
    local: bool,
    
    /// Analyze an image (specify image filename)
    #[arg(short, long)]
    image: Option<String>,
}

fn display_menu() {
    println!("\n=== Ollama Client Menu ===");
    println!("1. Generate Response (Remote Server)");
    println!("2. Generate Response (Local)");
    println!("3. Test Server Connection");
    println!("4. Test Local Connection");
    println!("5. View Configuration");
    println!("6. Analyze Image");
    println!("7. Exit");
    print!("Choose an option (1-7): ");
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
        Ok(ip) => println!("Remote Server IP: {}", ip),
        Err(_) => println!("Remote Server IP: Not set in .env file"),
    }
    
    match std::env::var("model") {
        Ok(model) => println!("Model: {}", model),
        Err(_) => println!("Model: llama3.2 (default)"),
    }
    
    println!("Local Server: http://localhost:11434");
    println!("Images Directory: ./images/");
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
    
    if let Some(image_file) = args.image {
        imagedescriber::analyze_specific_image(image_file).await?;
        return Ok(());
    }
    
    if args.local {
        if let Some(prompt) = args.prompt {
            connectlocally::generate_with_prompt(prompt).await?;
        } else {
            connectlocally::generate_response().await?;
        }
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
                match connectlocally::generate_response().await {
                    Ok(_) => println!("✅ Generation completed successfully!"),
                    Err(e) => println!("❌ Error: {}", e),
                }
            },
            "3" => {
                match connecttoollama::test_connection().await {
                    Ok(_) => {},
                    Err(e) => println!("❌ Error: {}", e),
                }
            },
            "4" => {
                match connectlocally::test_connection().await {
                    Ok(_) => {},
                    Err(e) => println!("❌ Error: {}", e),
                }
            },
            "5" => {
                display_config();
            },
            "6" => {
                match imagedescriber::analyze_image().await {
                    Ok(_) => println!("✅ Image analysis completed successfully!"),
                    Err(e) => println!("❌ Error: {}", e),
                }
            },
            "7" => {
                println!("👋 Goodbye!");
                break;
            },
            _ => {
                println!("❌ Invalid option. Please choose 1-7.");
            }
        }
        
        // Add a pause before showing menu again
        println!("\nPress Enter to continue...");
        let mut _input = String::new();
        io::stdin().read_line(&mut _input).unwrap();
    }
    
    Ok(())
}
