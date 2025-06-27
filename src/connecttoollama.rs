use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use tokio::io::{self, AsyncWriteExt};
use tokio_stream::StreamExt;
use std::io::{stdin, stdout, Write};

// Function to get user input with a prompt
pub fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    stdout().flush().unwrap();
    
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

// Main function to connect to Ollama and generate response
pub async fn generate_response() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Read server IP from .env file
    let server_ip = std::env::var("server_ip")
        .expect("server_ip must be set in .env file");
    
    // Read model from .env file
    let model = std::env::var("model")
        .unwrap_or_else(|_| "llama3.2".to_string());
    
    // Get prompt from user
    let user_prompt = get_user_input("Enter your prompt: ");
    
    // Construct the full server URL
    let server_url = format!("http://{}", server_ip);
    println!("Connecting to: {}:11434", server_url);
    println!("Using model: {}", model);
    
    // Create Ollama client
    let ollama = Ollama::new(server_url, 11434);
    
    // Create generation request
    let request = GenerationRequest::new(model, user_prompt);
    
    // Get streaming response
    let mut stream = ollama.generate_stream(request).await?;
    
    // Handle output
    let mut stdout = io::stdout();
    
    println!("\n--- Response ---");
    
    while let Some(res) = stream.next().await {
        let responses = res.unwrap();
        
        for resp in responses {
            stdout.write_all(resp.response.as_bytes()).await.unwrap();
            stdout.flush().await.unwrap();
        }
    }
    
    println!();
    Ok(())
}

// Function to generate response with custom prompt (non-interactive)
pub async fn generate_with_prompt(prompt: String) -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let server_ip = std::env::var("server_ip")
        .expect("server_ip must be set in .env file");
    
    let model = std::env::var("model")
        .unwrap_or_else(|_| "llama3.2".to_string());
    
    let server_url = format!("http://{}", server_ip);
    println!("Connecting to: {}:11434", server_url);
    println!("Using model: {}", model);
    
    let ollama = Ollama::new(server_url, 11434);
    let request = GenerationRequest::new(model, prompt);
    let mut stream = ollama.generate_stream(request).await?;
    let mut stdout = io::stdout();
    
    println!("\n--- Response ---");
    
    while let Some(res) = stream.next().await {
        let responses = res.unwrap();
        for resp in responses {
            stdout.write_all(resp.response.as_bytes()).await.unwrap();
            stdout.flush().await.unwrap();
        }
    }
    
    println!();
    Ok(())
}

// Function to test connection to server
pub async fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let server_ip = std::env::var("server_ip")
        .expect("server_ip must be set in .env file");
    
    let server_url = format!("http://{}", server_ip);
    println!("Testing connection to: {}:11434", server_url);
    
    let ollama = Ollama::new(server_url, 11434);
    let request = GenerationRequest::new("llama3.2".to_string(), "Hello".to_string());
    
    match ollama.generate_stream(request).await {
        Ok(_) => println!("✅ Connection successful!"),
        Err(e) => println!("❌ Connection failed: {}", e),
    }
    
    Ok(())
}
