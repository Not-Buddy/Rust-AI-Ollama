use ollama_rs::{Ollama, generation::completion::request::GenerationRequest, generation::images::Image};
use tokio::io::{self, AsyncWriteExt};
use tokio_stream::StreamExt;
use std::io::{stdin, stdout, Write};
use std::fs;
use std::path::Path;
use std::time::Instant;
use base64::{Engine as _, engine::general_purpose};

// Function to get user input with a prompt
pub fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    stdout().flush().unwrap();
    
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

// Function to list available images in the images directory
fn list_images() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let images_dir = Path::new("./images");
    
    if !images_dir.exists() {
        fs::create_dir_all(images_dir)?;
        println!("Created images directory: ./images/");
        println!("Please add some images to this directory and try again.");
        return Ok(vec![]);
    }
    
    let mut image_files = Vec::new();
    
    for entry in fs::read_dir(images_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp") {
                if let Some(filename) = path.file_name() {
                    image_files.push(filename.to_string_lossy().to_string());
                }
            }
        }
    }
    
    image_files.sort();
    Ok(image_files)
}

// Function to create Image object from file path
fn create_image_from_file(image_path: &Path) -> Result<Image, Box<dyn std::error::Error>> {
    let image_data = fs::read(image_path)?;
    let base64_string = general_purpose::STANDARD.encode(&image_data);
    
    // Create Image object with base64 data
    let image = Image::from_base64(&base64_string);
    Ok(image)
}

// Function to determine connection type (server first, then local fallback)
fn should_use_local() -> bool {
    dotenv::dotenv().ok();
    
    // Check if server_ip is set
    if let Ok(server_ip) = std::env::var("server_ip") {
        // If server_ip is explicitly set to localhost, use local
        if server_ip.contains("localhost") || server_ip.contains("127.0.0.1") {
            return true;
        }
        // If server_ip is set to a remote address, try server first
        return false;
    } else {
        // No server_ip set - still default to server (false) to try remote first
        // This will cause an error which can be caught and fallback to local
        return false;
    }
}


// Main function to analyze images interactively
pub async fn analyze_image() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Image Analysis ===");
    
    // List available images
    let image_files = list_images()?;
    
    if image_files.is_empty() {
        println!("No images found in ./images/ directory.");
        println!("Supported formats: jpg, jpeg, png, gif, bmp, webp");
        return Ok(());
    }
    
    // Display available images
    println!("Available images:");
    for (i, filename) in image_files.iter().enumerate() {
        println!("{}. {}", i + 1, filename);
    }
    
    // Get user selection
    let selection = get_user_input("\nSelect an image (enter number): ");
    let index: usize = selection.parse::<usize>()
        .map_err(|_| "Invalid selection")?
        .saturating_sub(1);
    
    if index >= image_files.len() {
        return Err("Invalid image selection".into());
    }
    
    let selected_image = &image_files[index];
    
    // Get custom prompt or use default
    let custom_prompt = get_user_input("Enter custom prompt (or press Enter for default description): ");
    let prompt = if custom_prompt.is_empty() {
        "Describe this image in detail.".to_string()
    } else {
        custom_prompt
    };
    
    analyze_image_with_prompt(selected_image, &prompt).await
}

// Function to analyze a specific image (for command line use)
pub async fn analyze_specific_image(filename: String) -> Result<(), Box<dyn std::error::Error>> {
    let prompt = "Describe this image in detail.";
    analyze_image_with_prompt(&filename, prompt).await
}

// Core function to analyze an image with a given prompt
async fn analyze_image_with_prompt(filename: &str, prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    // Load image and create Image object
    let image_path = Path::new("./images").join(filename);
    
    if !image_path.exists() {
        return Err(format!("Image file not found: {}", filename).into());
    }
    
    println!("Loading image: {}", filename);
    let image = create_image_from_file(&image_path)?;
    
    // Try server first, then fallback to local
    let use_local = should_use_local();
    
    let (ollama, connection_info) = if !use_local {
        // Try remote server first
        match std::env::var("server_ip") {
            Ok(server_ip) => {
                let server_url = format!("http://{}", server_ip);
                println!("Attempting to use remote server: {}:11434", server_url);
                (Ollama::new(server_url.clone(), 11434), format!("{}:11434", server_url))
            },
            Err(_) => {
                println!("No server_ip configured, falling back to local");
                (Ollama::new("http://localhost", 11434), "http://localhost:11434".to_string())
            }
        }
    } else {
        println!("Using local Ollama instance");
        (Ollama::new("http://localhost", 11434), "http://localhost:11434".to_string())
    };
    
    // Use a vision model (llava is common for image analysis)
    let model = std::env::var("vision_model")
        .unwrap_or_else(|_| "llava".to_string());
    
    println!("Using model: {}", model);
    println!("Analyzing image...");
    
    // Create the request with image
    let request = GenerationRequest::new(model.clone(), prompt.to_string())
        .images(vec![image.clone()]);
    
    // Start timing
    let start_time = Instant::now();
    
    // Try to get streaming response, with fallback logic
    let mut stream = match ollama.generate_stream(request).await {
        Ok(stream) => stream,
        Err(e) => {
            // If remote server failed and we weren't already using local, try local
            if !use_local && !connection_info.contains("localhost") {
                println!("❌ Remote server failed: {}", e);
                println!("🔄 Falling back to local Ollama instance...");
                
                let local_ollama = Ollama::new("http://localhost", 11434);
                let local_request = GenerationRequest::new(model, prompt.to_string())
                    .images(vec![image]);
                
                match local_ollama.generate_stream(local_request).await {
                    Ok(local_stream) => {
                        println!("✅ Connected to local Ollama");
                        local_stream
                    },
                    Err(local_e) => {
                        return Err(format!("Both remote and local connections failed. Remote: {}, Local: {}", e, local_e).into());
                    }
                }
            } else {
                return Err(e.into());
            }
        }
    };
    
    // Handle output
    let mut stdout = io::stdout();
    
    println!("\n--- Image Analysis ---");
    
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
    
    // Calculate tokens
    if eval_count > 0 {
        total_tokens = eval_count;
    } else {
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
    println!("Image: {}", filename);
    println!("Connection: {}", if connection_info.contains("localhost") { "Local" } else { "Remote" });
    println!("Total time: {:.2}s", elapsed_time.as_secs_f64());
    println!("Tokens generated: {}", total_tokens);
    println!("Tokens per second: {:.2}", tokens_per_sec);
    
    if eval_duration > 0 {
        let eval_time_sec = eval_duration as f64 / 1_000_000_000.0;
        let ollama_tokens_per_sec = if eval_time_sec > 0.0 {
            eval_count as f64 / eval_time_sec
        } else {
            0.0
        };
        println!("Ollama eval time: {:.2}s", eval_time_sec);
        println!("Ollama tokens/sec: {:.2}", ollama_tokens_per_sec);
    }
    
    println!("----------------------------");
    
    Ok(())
}


// Function to test if vision model is available
pub async fn test_vision_model() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let model = std::env::var("vision_model")
        .unwrap_or_else(|_| "llava".to_string());
    
    println!("Testing vision model: {}", model);
    
    let use_local = should_use_local();
    let ollama = if use_local {
        Ollama::new("http://localhost", 11434)
    } else {
        let server_ip = std::env::var("server_ip")
            .expect("server_ip must be set in .env file");
        let server_url = format!("http://{}", server_ip);
        Ollama::new(server_url, 11434)
    };
    
    // Test with a simple request (no image)
    let request = GenerationRequest::new(model, "Hello".to_string());
    
    match ollama.generate_stream(request).await {
        Ok(_) => println!("✅ Vision model is available!"),
        Err(e) => println!("❌ Vision model test failed: {}", e),
    }
    
    Ok(())
}
