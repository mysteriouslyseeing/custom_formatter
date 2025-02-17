use std::io::{stdout, Write};

use custom_formatter::{custom_format, DebugFormatter, DisplayFormatter};

fn main() {
    let a = custom_format!(with DebugFormatter, "hello {} world", "beautiful");
    let b = custom_format!(with DisplayFormatter, "hello {} world number {}", "beautiful", 2);

    println!("{a}"); // Prints hello "beautiful" world
    println!("{b}"); // Prints hello beautiful world number 2

    let c: Vec<u8> = custom_format!("hello world number {}\n", b'3');

    stdout().write(&c).unwrap(); // Prints hello world number 3
}
