// Import the Ollama client and request types for interacting with Ollama API
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
// Import async I/O traits and utilities from tokio for non-blocking I/O operations
use tokio::io::{self, AsyncWriteExt};  // AsyncWriteExt provides async write methods
// Import StreamExt trait to work with async streams (for handling streaming responses)
use tokio_stream::StreamExt;
// Import clap's Parser derive macro for command-line argument parsing
use clap::Parser;

// Define a struct to hold command-line arguments using clap's derive API
#[derive(Parser)]
struct Args {
    // Model name argument with short flag (-m) and long flag (--model)
    // Defaults to "llama3.2" if not provided
    #[arg(short, long, default_value = "llama3.2")]
    model: String,
    
    // Boolean flag to enable streaming mode with short (-s) and long (--stream) flags
    // This determines whether we get responses in real-time chunks or all at once
    #[arg(short, long)]
    stream: bool,
    
    // The actual prompt/question to send to the LLM (positional argument)
    prompt: String,
}

// Mark main function as async and use tokio runtime
// This allows us to use async/await syntax throughout the program
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments into our Args struct
    let args = Args::parse();
    
    // Create a new Ollama client with default configuration
    // This connects to Ollama server running on localhost:11434 by default
    let ollama = Ollama::default();
    
    // Check if user requested streaming mode
    if args.stream {
        // === STREAMING MODE ===
        
        // Create a generation request with the specified model and prompt
        let request = GenerationRequest::new(args.model, args.prompt);
        
        // Send the request and get back an async stream of responses
        // This returns chunks of the response as they're generated, not all at once
        let mut stream = ollama.generate_stream(request).await?;
        
        // Get a handle to stdout for writing output
        // Using tokio's async stdout instead of std::io::stdout for non-blocking I/O
        let mut stdout = io::stdout();
        
        // Process each chunk from the stream as it arrives
        while let Some(res) = stream.next().await {
            // Unwrap the result (Note: in production, you'd want better error handling)
            let responses = res.unwrap();
            
            // Each response can contain multiple response chunks, so iterate through them
            for resp in responses {
                // Write the response text directly to stdout as bytes
                // Using write_all ensures all bytes are written
                stdout.write_all(resp.response.as_bytes()).await.unwrap();
                
                // Immediately flush the output buffer to make text appear in real-time
                // Without this, output would be buffered and appear in chunks
                stdout.flush().await.unwrap();
            }
        }
        
        // Print a newline after streaming is complete for clean formatting
        println!();
        
    } else {
        // === NON-STREAMING MODE ===
        
        // Create the same generation request
        let request = GenerationRequest::new(args.model, args.prompt);
        
        // Send request and wait for complete response
        // This blocks until the entire response is generated
        let response = ollama.generate(request).await?;
        
        // Print the complete response all at once
        println!("{}", response.response);
    }
    
    // Return Ok if everything succeeded
    Ok(())
}
