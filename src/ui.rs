use std::io::Write;

use colored::Colorize;

pub fn shell_select(question: &str, options: &[String]) -> usize {
    println!("{}", question.yellow());
    for (i, option) in options.iter().enumerate() {
        println!("  {}: {}", i, option);
    }

    loop {
        print!("> ");
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
