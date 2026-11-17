//! Simple REPL to try the Pinyin engine from the terminal.
//!
//! Usage:
//! ```
//! cargo run --example repl -p tkey-core
//! ```
//! Then type characters one by one (or a whole string per line).
//! Press Enter to commit, Ctrl-D to quit.

use std::io::{self, BufRead, Write};
use tkey_core::{Config, Engine};

fn main() {
    let mut engine = Engine::new(Config::default());
    let stdin = io::stdin();
    let stdout = io::stdout();

    println!("TKey Pinyin REPL — type characters, Enter to commit, Ctrl-D to quit.");
    println!("Tip: type a full string like 'ni2hao3' then press Enter.\n");

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let mut last_preedit = String::new();

        for ch in line.chars() {
            let out = match ch {
                '\x08' => engine.backspace(), // Backspace char
                _ => engine.feed(ch),
            };
            if !out.commit.is_empty() {
                println!("  commit : {}", out.commit);
            }
            last_preedit = out.preedit.clone();
        }

        // Commit on Enter
        let out = engine.commit();
        if !out.commit.is_empty() {
            println!("  commit : {}", out.commit);
        } else if !last_preedit.is_empty() {
            println!("  (nothing to commit, preedit was: {})", last_preedit);
        }

        print!("preedit : {} > ", engine_preedit_display(&engine));
        stdout.lock().flush().ok();
    }

    println!("\nBye.");
}

fn engine_preedit_display(_engine: &Engine) -> &'static str {
    // After commit the preedit is always empty; show placeholder.
    ""
}
