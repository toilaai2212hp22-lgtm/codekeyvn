//! CodeKey terminal front-end: transform, REPL, demo.

use std::io::{self, BufRead, Write};

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use codekey_engine::{Engine, InputMethod, KeyResult};

#[derive(Parser, Debug)]
#[command(
    name = "codekey",
    version,
    about = "CodeKey — bộ gõ tiếng Việt (Telex/VNI) cho terminal & Linux IME"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input method (default: telex)
    #[arg(short, long, global = true, default_value = "telex")]
    method: MethodArg,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Transform one string: codekey transform "xin chaof"
    Transform {
        text: String,
    },
    /// Read stdin lines, write transformed stdout
    Batch,
    /// Interactive REPL (type Telex/VNI, see Vietnamese)
    Repl,
    /// Print version / engine info
    Info,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum MethodArg {
    Telex,
    Vni,
}

impl From<MethodArg> for InputMethod {
    fn from(m: MethodArg) -> Self {
        match m {
            MethodArg::Telex => InputMethod::Telex,
            MethodArg::Vni => InputMethod::Vni,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let method = InputMethod::from(cli.method);

    match cli.command.unwrap_or(Commands::Repl) {
        Commands::Transform { text } => {
            println!("{}", Engine::transform(method, &text));
        }
        Commands::Batch => {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let line = line?;
                println!("{}", Engine::transform(method, &line));
            }
        }
        Commands::Repl => run_repl(method)?,
        Commands::Info => {
            println!("CodeKey {}", codekey_engine::VERSION);
            println!("Methods: telex, vni");
            println!("IME: IBus (codekey-ibus), Fcitx5 (addon)");
            println!("Example: codekey transform \"Vieejt Nam\"");
        }
    }
    Ok(())
}

fn run_repl(method: InputMethod) -> Result<()> {
    let mut eng = Engine::new(method);
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    writeln!(
        stdout,
        "CodeKey REPL — method={method}  |  :q quit  :m telex|vni  :c clear  :t toggle"
    )?;
    writeln!(
        stdout,
        "Gõ Telex/VNI rồi Enter để xem kết quả cả dòng (batch), hoặc dùng chế độ live bên dưới."
    )?;
    writeln!(stdout, "Live mode: gõ từng từ, Space commit. Ctrl-D thoát.\n")?;
    stdout.flush()?;

    // Line-oriented transform (easiest for terminal)
    for line in stdin.lock().lines() {
        let line = line?;
        let t = line.trim();
        if t == ":q" || t == ":quit" || t == "quit" {
            break;
        }
        if t == ":c" || t == ":clear" {
            eng.reset();
            continue;
        }
        if t == ":t" || t == ":toggle" {
            let on = eng.toggle();
            writeln!(stdout, "[enabled={on}]")?;
            continue;
        }
        if let Some(rest) = t.strip_prefix(":m ") {
            if let Some(m) = InputMethod::parse(rest.trim()) {
                eng.set_method(m);
                writeln!(stdout, "[method={m}]")?;
            } else {
                writeln!(stdout, "unknown method, use telex|vni")?;
            }
            continue;
        }
        if t == ":live" {
            run_live(&mut eng, &mut stdout)?;
            continue;
        }

        let out = Engine::transform(eng.method(), &line);
        writeln!(stdout, "→ {out}")?;
        stdout.flush()?;
    }
    Ok(())
}

/// Character-at-a-time live composition (shows preedit).
fn run_live(eng: &mut Engine, stdout: &mut impl Write) -> Result<()> {
    use std::io::Read;
    writeln!(
        stdout,
        "LIVE (raw terminal). Gõ bình thường, Esc hoặc Ctrl-C để về REPL."
    )?;
    stdout.flush()?;

    // Best-effort without crossterm: read bytes one by one if tty
    let mut stdin = io::stdin();
    let mut buf = [0u8; 1];
    let mut line_out = String::new();

    loop {
        let n = stdin.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let b = buf[0];
        if b == 0x1b || b == 3 {
            // Esc / Ctrl-C
            break;
        }
        if b == b'\n' || b == b'\r' {
            let rest = eng.commit_text();
            line_out.push_str(&rest);
            writeln!(stdout, "\n✓ {line_out}")?;
            line_out.clear();
            stdout.flush()?;
            continue;
        }
        if b == 0x7f || b == 0x08 {
            match eng.backspace() {
                KeyResult::Backspace => {
                    // redraw simple
                    write!(stdout, "\r\x1b[K{}", eng.preedit())?;
                    stdout.flush()?;
                }
                KeyResult::CommitAndPass => {
                    line_out.pop();
                    write!(stdout, "\r\x1b[K{line_out}")?;
                    stdout.flush()?;
                }
                _ => {}
            }
            continue;
        }
        let ch = b as char;
        if !ch.is_ascii() {
            continue;
        }
        match eng.feed(ch) {
            KeyResult::Update | KeyResult::Append => {
                write!(stdout, "\r\x1b[K{line_out}{}", eng.preedit())?;
                stdout.flush()?;
            }
            KeyResult::Commit => {
                line_out.push_str(&eng.commit_text());
                line_out.push(ch);
                write!(stdout, "\r\x1b[K{line_out}")?;
                stdout.flush()?;
            }
            KeyResult::CommitAndPass => {
                line_out.push_str(&eng.commit_text());
                line_out.push(ch);
                write!(stdout, "\r\x1b[K{line_out}")?;
                stdout.flush()?;
            }
            KeyResult::Backspace | KeyResult::Ignored => {}
        }
    }
    writeln!(stdout, "\n(back to line mode)")?;
    Ok(())
}
