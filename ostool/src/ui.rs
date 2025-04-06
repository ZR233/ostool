use std::io::Write;

use colored::Colorize;

pub fn shell_select<T: AsRef<str>>(question: &str, options: &[T]) -> usize {
    println!("{}", question.yellow());
    for (i, option) in options.iter().enumerate() {
        println!("  {}: {}", i, option.as_ref());
    }

    loop {
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        let n = std::io::stdin().read_line(&mut input).unwrap();
        if n == 0 {
            continue;
        }

        match input.trim().parse::<usize>() {
            Ok(n) => {
                if n < options.len() {
                    return n;
                } else {
                    println!("Invalid input");
                }
            }
            Err(_) => {
                println!("Invalid input");
            }
        }
    }
}

pub trait InputParser {
    fn parse(input: &str) -> Result<Self, String>
    where
        Self: Sized;
}

impl InputParser for String {
    fn parse(input: &str) -> Result<Self, String> {
        Ok(input.to_string())
    }
}

impl InputParser for bool {
    fn parse(input: &str) -> Result<Self, String> {
        match input.trim() {
            "y" | "Y" | "yes" | "Yes" | "YES" => Ok(true),
            "n" | "N" | "no" | "No" | "NO" => Ok(false),
            _ => Err("Invalid input".to_string()),
        }
    }
}

impl InputParser for usize {
    fn parse(input: &str) -> Result<Self, String> {
        match input.trim().parse::<usize>() {
            Ok(n) => Ok(n),
            Err(_) => Err("Invalid input".to_string()),
        }
    }
}

pub fn shell_input<T: InputParser>(question: &str) -> T {
    println!("{}", question.yellow());
    loop {
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        match T::parse(&input) {
            Ok(n) => {
                return n;
            }
            Err(e) => {
                println!("{}", e.red());
            }
        }
    }
}
