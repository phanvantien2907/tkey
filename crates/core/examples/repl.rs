//! Simple REPL to try the Pinyin engine from the terminal.
//!
//! Usage:
//! ```
//! cargo run --example repl -p tkey-core -- [FLAGS]
//! ```
//! Flags:
//!   --sandhi[=rules|off]       Turn tone sandhi on/off (default: off; bare --sandhi turns it on)
//!   --blocklist[=true|false]   Turn sandhi blocklist on/off (default: false; bare --blocklist turns it on)
//!   --no-blocklist             Explicitly disable sandhi blocklist
//!   --trim                     Strip leading and trailing whitespace of committed text
//!   --collapse                 Collapse multiple spaces between words into one space
//!   --show-spaces              Render space as ·, tab as →, NBSP as ⍽
//!
//! Environment variables:
//!   TONE_SANDHI=rules|off
//!   SANDHI_BLOCKLIST=1|0|true|false

use std::io::{self, BufRead, Write};
use tkey_core::config::ToneSandhi;
use tkey_core::{Config, Engine};

fn main() {
    let mut tone_sandhi = match std::env::var("TONE_SANDHI") {
        Ok(v) if v.eq_ignore_ascii_case("rules") || v == "1" => ToneSandhi::Rules,
        _ => ToneSandhi::Off,
    };
    let mut sandhi_blocklist = matches!(
        std::env::var("SANDHI_BLOCKLIST"),
        Ok(v) if v.eq_ignore_ascii_case("true") || v == "1"
    );
    let mut flag_trim = false;
    let mut flag_collapse = false;
    let mut flag_show_spaces = false;

    for arg in std::env::args().skip(1) {
        if arg == "--sandhi" || arg == "--sandhi=rules" {
            tone_sandhi = ToneSandhi::Rules;
        } else if arg == "--sandhi=off" {
            tone_sandhi = ToneSandhi::Off;
        } else if arg == "--blocklist" || arg == "--blocklist=true" {
            sandhi_blocklist = true;
        } else if arg == "--blocklist=false" || arg == "--no-blocklist" {
            sandhi_blocklist = false;
        } else if arg == "--trim" {
            flag_trim = true;
        } else if arg == "--collapse" {
            flag_collapse = true;
        } else if arg == "--show-spaces" {
            flag_show_spaces = true;
        } else if arg == "--help" || arg == "-h" {
            println!("Usage: cargo run --example repl -p tkey-core -- [FLAGS]");
            println!("\nFlags:");
            println!("  --sandhi[=rules|off]       Turn tone sandhi on/off (default: off)");
            println!("  --blocklist[=true|false]   Turn sandhi blocklist on/off (default: false)");
            println!("  --no-blocklist             Disable sandhi blocklist");
            println!("  --trim                     Strip leading/trailing whitespace");
            println!("  --collapse                 Collapse runs of spaces into one");
            println!("  --show-spaces              Render space as ·, tab as →, NBSP as ⍽");
            println!("\nEnvironment variables:");
            println!("  TONE_SANDHI=rules|off      (default: off)");
            println!("  SANDHI_BLOCKLIST=1|0       (default: 0)");
            return;
        }
    }

    let config = Config {
        tone_sandhi,
        sandhi_blocklist,
        ..Default::default()
    };

    let mut engine = Engine::new(config.clone());
    let stdin = io::stdin();
    let stdout = io::stdout();

    println!("============================================================");
    println!("TKey Pinyin REPL");
    println!("  Mode             : Pinyin");
    println!("  Tone Sandhi      : {:?}", config.tone_sandhi);
    println!("  Sandhi Blocklist : {}", config.sandhi_blocklist);
    println!("  Trim Whitespace  : {}", flag_trim);
    println!("  Collapse Spaces  : {}", flag_collapse);
    println!("  Show Spaces      : {}", flag_show_spaces);
    println!("============================================================");
    println!("Options & flags:");
    println!("  --trim          : Strip leading/trailing whitespace");
    println!("  --collapse      : Collapse multiple spaces between words");
    println!("  --show-spaces   : Render space as ·, tab as →, NBSP as ⍽");
    println!("  --sandhi        : Turn on tone sandhi rules");
    println!("  --blocklist     : Turn on sandhi blocklist (protects 部队, 衣服)");
    println!("============================================================\n");
    println!(
        "Type characters one by one (or full line e.g. 'wo1   zhong1guo2'), Enter to commit, Ctrl-D to quit.\n"
    );

    print!("> ");
    stdout.lock().flush().ok();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        // Feed the typed line through the shared feed_str helper
        let (mut committed, _) = engine.feed_str(&line);
        // Commit on Enter at the end of the line
        let out = engine.commit();
        committed.push_str(&out.commit);

        let mut display = committed;
        if flag_collapse {
            display = collapse_spaces(&display);
        }
        if flag_trim {
            display = display.trim().to_string();
        }
        if flag_show_spaces {
            display = show_spaces(&display);
        }

        println!("  commit  : {}", display);
        println!("  preedit : {}", engine.preedit());

        print!("> ");
        stdout.lock().flush().ok();
    }

    println!("\nBye.");
}

fn collapse_spaces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_space = false;
    for ch in s.chars() {
        if ch == ' ' {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(ch);
            last_was_space = false;
        }
    }
    result
}

fn show_spaces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            ' ' => result.push('·'),
            '\t' => result.push('→'),
            '\u{00A0}' => result.push('⍽'),
            '\u{3000}' => result.push('␣'),
            _ => result.push(ch),
        }
    }
    result
}
