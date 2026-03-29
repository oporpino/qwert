use super::colors::*;

const TICK: &str = "✓";
const ARROW: &str = "→";
const CROSS: &str = "✗";
const BULLET: &str = "•";

fn is_tty() -> bool {
    #[cfg(unix)]
    {
        extern "C" { fn isatty(fd: i32) -> i32; }
        unsafe { isatty(1) != 0 }
    }
    #[cfg(not(unix))]
    { false }
}

fn use_color() -> bool {
    is_tty() && std::env::var("NO_COLOR").is_err()
}

fn colorize(color: &str, text: &str) -> String {
    if use_color() {
        format!("{}{}{}", color, text, RESET)
    } else {
        text.to_string()
    }
}

// --- Status line printers ---

/// ✓  tool_name   message
pub fn ok(name: &str, msg: &str) {
    let tick = colorize(SUCCESS, TICK);
    let name_col = colorize(BOLD_WHITE, &format!("{:<12}", name));
    let msg_col = colorize(DIM, msg);
    println!("  {}  {}  {}", tick, name_col, msg_col);
}

/// →  tool_name   message
pub fn installing(name: &str, msg: &str) {
    let arrow = colorize(INFO, ARROW);
    let name_col = colorize(BOLD_WHITE, &format!("{:<12}", name));
    println!("  {}  {}  {}", arrow, name_col, msg);
}

/// ✗  tool_name   message
pub fn failed(name: &str, msg: &str) {
    let cross = colorize(ERROR, CROSS);
    let name_col = colorize(BOLD_WHITE, &format!("{:<12}", name));
    let msg_col = colorize(ERROR, msg);
    println!("  {}  {}  {}", cross, name_col, msg_col);
}

// --- Headings ---

pub fn h1(title: &str) {
    if use_color() {
        println!("\n{}{}{}", BOLD_WHITE, title, RESET);
        println!("{}{}{}", DIM, "─".repeat(title.len()), RESET);
    } else {
        println!("\n{}", title);
        println!("{}", "─".repeat(title.len()));
    }
}

pub fn h2(title: &str) {
    if use_color() {
        println!("\n  {}{}{}", BOLD_WHITE, title, RESET);
    } else {
        println!("\n  {}", title);
    }
}

// --- Info lines ---

pub fn info(msg: &str) {
    println!("  {}", colorize(INFO, msg));
}

pub fn warning(msg: &str) {
    println!("  {}", colorize(WARNING, &format!("warning: {}", msg)));
}

pub fn error(msg: &str) {
    eprintln!("  {}", colorize(ERROR, &format!("error: {}", msg)));
}

pub fn bullet(msg: &str) {
    println!("  {}  {}", colorize(DIM, BULLET), msg);
}

/// "  use <tool>      declare a tool for this machine"
pub fn command(cmd: &str, description: &str) {
    println!("  {}  {}", colorize(BOLD_WHITE, &format!("{:<30}", cmd)), colorize(DIM, description));
}

// --- Summary line ---

/// "  5/6 done  •  1 failed"
pub fn summary(done: usize, total: usize, failed: usize) {
    println!();
    let done_str = colorize(SUCCESS, &format!("{}/{} done", done, total));
    if failed > 0 {
        let fail_str = colorize(ERROR, &format!("{} failed", failed));
        println!("  {}  {}  {}", done_str, colorize(DIM, BULLET), fail_str);
    } else {
        println!("  {}", done_str);
    }
    println!();
}

// --- Search result ---

/// "  neovim    [brew]    Neovim text editor    v0.10.2"
pub fn search_result(name: &str, kind: &str, description: &str, version: Option<&str>) {
    let name_col = colorize(BOLD_WHITE, &format!("{:<12}", name));
    let kind_col = match kind {
        "brew" => colorize(BRIGHT_BLUE, &format!("[{:<5}]", kind)),
        _ => colorize(BRIGHT_YELLOW, &format!("[{:<5}]", kind)),
    };
    let ver = version
        .map(|v| colorize(DIM, &format!("  {}", v)))
        .unwrap_or_default();
    println!("  {}  {}  {}{}", name_col, kind_col, description, ver);
}

// --- Blank line ---
pub fn blank() {
    println!();
}
