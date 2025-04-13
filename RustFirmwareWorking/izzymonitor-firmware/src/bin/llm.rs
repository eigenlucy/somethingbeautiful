use anyhow::Result;
use log::info;
use crate::network;

// Process a user query through the LLM backend
pub async fn process_query(query: &str) -> Result<String> {
    info!("Processing user query through LLM: {}", query);
    
    // Send query to backend
    let response = network::query_llm_backend(query)?;
    
    // Parse and handle the response
    // In a real application, you would parse JSON or whatever format your backend returns
    
    Ok(response)
}

// Process a navigation request
pub fn get_navigation_directions(source: &str, destination: &str) -> Result<String> {
    info!("Getting navigation directions from {} to {}", source, destination);
    
    // Get directions from the backend
    let trip_data = network::get_trip_directions(source, destination)?;
    
    // Format the directions into a user-friendly response
    let mut response = format!(
        "From {} to {}\nDistance: {}\nTime
