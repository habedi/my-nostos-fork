//! REPL Connect Client - connect to a running TUI REPL server
//!
//! Usage: `nostos connect -p <port>`
//!
//! Connects to a TUI REPL server started with `nostos repl --serve <port>`
//! and provides a line-based interface to send commands.

use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonically increasing command ID
static COMMAND_ID: AtomicU64 = AtomicU64::new(1);

/// Parse command-line arguments for connect
pub fn run_connect(args: &[String]) -> ExitCode {
    let mut port: Option<u16> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--port" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u16>() {
                        Ok(p) => port = Some(p),
                        Err(_) => {
                            eprintln!("Error: Invalid port number '{}'", args[i + 1]);
                            return ExitCode::FAILURE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: -p requires a port number");
                    return ExitCode::FAILURE;
                }
            }
            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            _ => {
                // Try to parse as port number directly
                if port.is_none() {
                    if let Ok(p) = args[i].parse::<u16>() {
                        port = Some(p);
                    } else {
                        eprintln!("Error: Unknown argument '{}'", args[i]);
                        return ExitCode::FAILURE;
                    }
                }
                i += 1;
            }
        }
    }

    let port = match port {
        Some(p) => p,
        None => {
            eprintln!("Error: Port number required");
            eprintln!("Usage: nostos connect -p <port>");
            return ExitCode::FAILURE;
        }
    };

    connect_to_server(port)
}

fn print_help() {
    eprintln!("Connect to a running REPL server");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    nostos connect -p <port>");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    -p, --port <PORT>    Port to connect to");
    eprintln!("    -h, --help           Show this help");
    eprintln!();
    eprintln!("COMMANDS (after connecting):");
    eprintln!("    :load <file>         Load a .nos file or directory");
    eprintln!("    :reload              Reload all loaded files");
    eprintln!("    :status              Show compilation status");
    eprintln!("    :eval <expr>         Evaluate an expression");
    eprintln!("    :compile <file>      Compile a file (check for errors)");
    eprintln!("    :quit                Disconnect from server");
    eprintln!();
    eprintln!("EXAMPLE:");
    eprintln!("    # Terminal 1: Start REPL with server");
    eprintln!("    nostos repl --serve 7878");
    eprintln!();
    eprintln!("    # Terminal 2: Connect to it");
    eprintln!("    nostos connect -p 7878");
}

fn connect_to_server(port: u16) -> ExitCode {
    let addr = format!("127.0.0.1:{}", port);

    let stream = match TcpStream::connect(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Could not connect to {}: {}", addr, e);
            eprintln!("Make sure the REPL server is running with: nostos repl --serve {}", port);
            return ExitCode::FAILURE;
        }
    };

    eprintln!("Connected to REPL server at {}", addr);
    eprintln!("Type :help for commands, :quit to disconnect");
    eprintln!();

    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));
    let mut writer = stream;

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // Print prompt
        print!("nostos> ");
        stdout.flush().ok();

        // Read user input
        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                // EOF
                eprintln!("\nDisconnected.");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Handle local commands
        if line == ":quit" || line == ":q" || line == ":exit" {
            eprintln!("Disconnected.");
            break;
        }

        if line == ":help" || line == ":h" || line == "?" {
            print_client_help();
            continue;
        }

        // Parse and send command
        let (cmd, args) = parse_input(line);
        let json = format_command(&cmd, &args);

        // Send to server
        if let Err(e) = writeln!(writer, "{}", json) {
            eprintln!("Error sending command: {}", e);
            break;
        }
        writer.flush().ok();

        // Read response
        let mut response = String::new();
        match reader.read_line(&mut response) {
            Ok(0) => {
                eprintln!("Server disconnected.");
                break;
            }
            Ok(_) => {
                print_response(&response);
            }
            Err(e) => {
                eprintln!("Error reading response: {}", e);
                break;
            }
        }
    }

    ExitCode::SUCCESS
}

fn print_client_help() {
    eprintln!("Commands:");
    eprintln!("  :load <path>    Load a .nos file or directory");
    eprintln!("  :reload         Reload all loaded files");
    eprintln!("  :status         Show compilation status");
    eprintln!("  :eval <expr>    Evaluate an expression");
    eprintln!("  :compile <file> Compile a file and show errors");
    eprintln!("  :quit           Disconnect from server");
    eprintln!("  :help           Show this help");
    eprintln!();
    eprintln!("You can also type code directly to evaluate it.");
}

/// Parse user input into command and arguments
fn parse_input(line: &str) -> (String, String) {
    if line.starts_with(':') {
        // Command
        let parts: Vec<&str> = line[1..].splitn(2, ' ').collect();
        let cmd = parts[0].to_string();
        let args = if parts.len() > 1 { parts[1].to_string() } else { String::new() };
        (cmd, args)
    } else {
        // Direct code evaluation
        ("eval".to_string(), line.to_string())
    }
}

/// Format a command as JSON for the server
fn format_command(cmd: &str, args: &str) -> String {
    let id = COMMAND_ID.fetch_add(1, Ordering::SeqCst);

    // Determine the appropriate key for the args
    let arg_key = match cmd {
        "load" | "compile" => "file",
        "eval" => "code",
        _ => "args",
    };

    // Escape the args for JSON
    let escaped_args = escape_json_string(args);

    if args.is_empty() {
        format!(r#"{{"id":{},"cmd":"{}"}}"#, id, cmd)
    } else {
        format!(r#"{{"id":{},"cmd":"{}","{}":"{}"}}"#, id, cmd, arg_key, escaped_args)
    }
}

/// Escape a string for JSON
fn escape_json_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

/// Parse and print a JSON response from the server
fn print_response(json: &str) {
    // Simple JSON parsing without serde
    let json = json.trim();
    if !json.starts_with('{') || !json.ends_with('}') {
        eprintln!("Invalid response: {}", json);
        return;
    }

    // Extract fields manually
    let status = extract_json_field(json, "status");
    let output = extract_json_field(json, "output");
    let errors = extract_json_array(json, "errors");

    // Print based on status
    match status.as_str() {
        "ok" => {
            if !output.is_empty() {
                println!("{}", unescape_json_string(&output));
            }
        }
        "error" => {
            eprintln!("Error: {}", unescape_json_string(&output));
            if !errors.is_empty() {
                for error in &errors {
                    let file = extract_json_field(error, "file");
                    let line = extract_json_field(error, "line");
                    let message = extract_json_field(error, "message");
                    eprintln!("  {}:{}: {}", file, line, unescape_json_string(&message));
                }
            }
        }
        _ => {
            println!("{}", unescape_json_string(&output));
        }
    }
}

/// Extract a string field from JSON (simple parser)
fn extract_json_field(json: &str, field: &str) -> String {
    let pattern = format!(r#""{}":"#, field);
    if let Some(start) = json.find(&pattern) {
        let rest = &json[start + pattern.len()..];
        // Check if value is a string (starts with ")
        if rest.starts_with('"') {
            // Find the closing quote, handling escaped quotes
            let mut end = 1;
            let chars: Vec<char> = rest.chars().collect();
            while end < chars.len() {
                if chars[end] == '"' && (end == 0 || chars[end - 1] != '\\') {
                    break;
                }
                end += 1;
            }
            return rest[1..end].to_string();
        }
        // Numeric value
        let end = rest.find(|c| c == ',' || c == '}').unwrap_or(rest.len());
        return rest[..end].to_string();
    }
    String::new()
}

/// Extract an array field from JSON (simple parser)
fn extract_json_array(json: &str, field: &str) -> Vec<String> {
    let pattern = format!(r#""{}":["#, field);
    if let Some(start) = json.find(&pattern) {
        let rest = &json[start + pattern.len()..];
        if let Some(end) = rest.find(']') {
            let array_content = &rest[..end];
            // Split by },{ to get individual objects
            let mut result = Vec::new();
            let mut depth = 0;
            let mut current = String::new();
            for c in array_content.chars() {
                match c {
                    '{' => {
                        depth += 1;
                        current.push(c);
                    }
                    '}' => {
                        depth -= 1;
                        current.push(c);
                        if depth == 0 {
                            result.push(current.clone());
                            current.clear();
                        }
                    }
                    ',' if depth == 0 => {
                        // Skip comma between objects
                    }
                    _ => {
                        if depth > 0 {
                            current.push(c);
                        }
                    }
                }
            }
            return result;
        }
    }
    Vec::new()
}

/// Unescape JSON string
fn unescape_json_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('u') => {
                    // Unicode escape \uXXXX
                    let mut hex = String::new();
                    for _ in 0..4 {
                        if let Some(h) = chars.next() {
                            hex.push(h);
                        }
                    }
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                    }
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}
