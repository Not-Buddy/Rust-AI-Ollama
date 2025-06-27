use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use tokio::io::{self, AsyncWriteExt};
use tokio_stream::StreamExt;
use std::time::Instant;
use std::io::{stdin, stdout, Write};

// Function to get user input with a prompt
pub fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    stdout().flush().unwrap();
    
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

pub async fn generate_response() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file for model preference
    dotenv::dotenv().ok();
    
    // Use local model from .env or default
    let model = std::env::var("model")
        .unwrap_or_else(|_| "llama3.2".to_string());
    
    // Get prompt from user
    let user_prompt = get_user_input("Enter your prompt: ");
    
    println!("Connecting to: http://localhost:11434");
    println!("Using model: {}", model);
    
    // Create Ollama client for localhost
    let ollama = Ollama::new("http://localhost", 11434);
    
    // Create generation request
    let request = GenerationRequest::new(model, user_prompt);
    
    // Start timing
    let start_time = Instant::now();
    
    // Get streaming response
    let mut stream = ollama.generate_stream(request).await?;
    
    // Handle output
    let mut stdout = io::stdout();
    
    println!("\n--- Response ---");
    
    // Variables to track metrics
    let mut total_tokens = 0;
    let mut response_text = String::new();
    let mut eval_count = 0;
    let mut eval_duration = 0;
    let mut total_duration = 0;
    
    while let Some(res) = stream.next().await {
        let responses = res.unwrap();
        
        for resp in responses {
            // Write the response text
            stdout.write_all(resp.response.as_bytes()).await.unwrap();
            stdout.flush().await.unwrap();
            
            // Collect response text for token counting
            response_text.push_str(&resp.response);
            
            // If this is the final response, it contains metrics
            if resp.done {
                eval_count = resp.eval_count.unwrap_or(0);
                eval_duration = resp.eval_duration.unwrap_or(0);
                total_duration = resp.total_duration.unwrap_or(0);
            }
        }
    }
    
    // Calculate elapsed time
    let elapsed_time = start_time.elapsed();
    
    // Calculate tokens (use eval_count if available, otherwise estimate from text)
    if eval_count > 0 {
        total_tokens = eval_count;
    } else {
        // Rough estimate: split by whitespace and count
        total_tokens = response_text.split_whitespace().count() as u64;
    }
    
    // Calculate tokens per second
    let tokens_per_sec = if elapsed_time.as_secs_f64() > 0.0 {
        total_tokens as f64 / elapsed_time.as_secs_f64()
    } else {
        0.0
    };
    
    // Display metrics
    println!("\n--- Performance Metrics ---");
    println!("Total time: {:.2}s", elapsed_time.as_secs_f64());
    println!("Tokens generated: {}", total_tokens);
    println!("Tokens per second: {:.2}", tokens_per_sec);
    
    // If we have detailed timing from Ollama
    if eval_duration > 0 {
        let eval_time_sec = eval_duration as f64 / 1_000_000_000.0; // Convert nanoseconds to seconds
        let ollama_tokens_per_sec = if eval_time_sec > 0.0 {
            eval_count as f64 / eval_time_sec
        } else {
            0.0
        };
        println!("Ollama eval time: {:.2}s", eval_time_sec);
        println!("Ollama tokens/sec: {:.2}", ollama_tokens_per_sec);
    }
    
    if total_duration > 0 {
        let total_time_sec = total_duration as f64 / 1_000_000_000.0;
        println!("Ollama total time: {:.2}s", total_time_sec);
    }
    
    println!("----------------------------");
    
    Ok(())
}

// Function to generate response with custom prompt (non-interactive)
pub async fn generate_with_prompt(prompt: String) -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let model = std::env::var("model")
        .unwrap_or_else(|_| "llama3.2".to_string());
    
    println!("Connecting to: http://localhost:11434");
    println!("Using model: {}", model);
    
    let ollama = Ollama::new("http://localhost", 11434);
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

// Function to test connection to local server
pub async fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing connection to: http://localhost:11434");
    
    let ollama = Ollama::new("http://localhost", 11434);
    let request = GenerationRequest::new("llama3.2".to_string(), "Hello".to_string());
    
    match ollama.generate_stream(request).await {
        Ok(_) => println!("✅ Local connection successful!"),
        Err(e) => println!("❌ Local connection failed: {}", e),
    }
    
    Ok(())
}

// Function to check available models locally
pub async fn list_models() -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking available models on localhost...");
    
    let ollama = Ollama::new("http://localhost", 11434);
    
    // Note: You might need to check ollama-rs documentation for the correct method
    // This is a placeholder - adjust based on the actual API
    match ollama.list_local_models().await {
        Ok(models) => {
            println!("Available models:");
            for model in models {
                println!("  - {}", model.name);
            }
        },
        Err(e) => println!("❌ Could not list models: {}", e),
    }
    
    Ok(())
}
