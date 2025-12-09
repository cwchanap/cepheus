use cepheus_lib::state::HistoryBuffer;
use cepheus_lib::models::OutputLine;

fn main() {
    // Test to reproduce the capacity bug
    let buffer = HistoryBuffer::new(3);
    
    // Fill buffer to exactly capacity
    for i in 0..3 {
        buffer.push(OutputLine::Stdout {
            text: format!("line{}", i),
            timestamp: i as u64,
        });
    }
    
    println!("Buffer length after filling to capacity: {}", buffer.len());
    println!("Expected: 3, Actual: {}", buffer.len());
    
    // Add one more line - this should trigger truncation + warning
    buffer.push(OutputLine::Stdout {
        text: "line3".to_string(),
        timestamp: 3,
    });
    
    println!("Buffer length after adding 4th line: {}", buffer.len());
    println!("Expected: 3, Actual: {}", buffer.len());
    println!("Has truncation warning: {}", buffer.has_truncation_warning());
    
    // The buffer should never exceed its configured capacity
    if buffer.len() > 3 {
        println!("BUG DETECTED: Buffer exceeds capacity!");
        println!("Capacity: 3, Actual length: {}", buffer.len());
    }
}