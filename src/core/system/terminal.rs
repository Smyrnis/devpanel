use super::shell_quote;
use crate::core::paths;

pub fn open_terminal_at(path: &str) {
    let Some(term) = find_terminal() else { return };
    let cd_cmd = format!("cd {} && exec bash", shell_quote(path));
    let result = match term.as_str() {
        "gnome-terminal" => std::process::Command::new("gnome-terminal")
            .arg("--working-directory")
            .arg(path)
            .spawn(),
        "xfce4-terminal" => std::process::Command::new("xfce4-terminal")
            .arg("--working-directory")
            .arg(path)
            .spawn(),
        "konsole" => std::process::Command::new("konsole")
            .arg("--workdir")
            .arg(path)
            .spawn(),
        "mate-terminal" => std::process::Command::new("mate-terminal")
            .arg("--working-directory")
            .arg(path)
            .spawn(),
        "xterm" => std::process::Command::new("xterm")
            .args(["-e", "bash", "-c", &cd_cmd])
            .spawn(),
        "lxterminal" | "tilix" => std::process::Command::new(&term)
            .arg("-e")
            .arg(format!("bash -c {}", shell_quote(&cd_cmd)))
            .spawn(),
        _ => std::process::Command::new(&term)
            .args(["-e", "bash", "-c", &cd_cmd])
            .spawn(),
    };
    let _ = result;
}

pub fn open_in_editor(editor_command: &str, path: &str) -> Result<(), String> {
    let command = editor_command.trim();
    if command.is_empty() {
        return Err("Editor command is empty. Set it in Config.".into());
    }
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("{} {}", command, shell_quote(path)))
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to open editor: {}", e))
}

pub fn open_db_terminal(binary: &str, socket_auth: bool) -> Result<String, String> {
    let mysql_cmd = if socket_auth {
        format!("sudo {} -u root", binary)
    } else {
        format!("sudo {} -u root -p", binary)
    };
    let inner = format!(
        "{cmd}; printf '\\n\\033[0;33m--- session ended, press Enter to close ---\\033[0m\\n'; read _",
        cmd = mysql_cmd,
    );
    let term = find_terminal().ok_or_else(|| {
        "No terminal emulator found. Install gnome-terminal, xterm, konsole, or xfce4-terminal."
            .to_string()
    })?;
    let result = match term.as_str() {
        "gnome-terminal" | "mate-terminal" => std::process::Command::new(&term)
            .args(["--", "bash", "-c", &inner])
            .spawn(),
        "xterm" | "konsole" => std::process::Command::new(&term)
            .args(["-e", "bash", "-c", &inner])
            .spawn(),
        "xfce4-terminal" => std::process::Command::new("xfce4-terminal")
            .arg("--command")
            .arg(format!("bash -c {}", shell_quote(&inner)))
            .spawn(),
        "tilix" => std::process::Command::new("tilix")
            .arg("-e")
            .arg(format!("bash -c {}", shell_quote(&inner)))
            .spawn(),
        "lxterminal" => std::process::Command::new("lxterminal")
            .arg("-e")
            .arg(format!("bash -c {}", shell_quote(&inner)))
            .spawn(),
        "x-terminal-emulator" => std::process::Command::new("x-terminal-emulator")
            .args(["-e", "bash", "-c", &inner])
            .spawn(),
        _ => std::process::Command::new(&term)
            .args(["-e", "bash", "-c", &inner])
            .spawn(),
    };
    match result {
        Ok(_) => Ok(format!("Launched '{}' in {}", mysql_cmd, term)),
        Err(e) => Err(format!("Failed to open {}: {}", term, e)),
    }
}

fn find_terminal() -> Option<String> {
    const CANDIDATES: &[&str] = &[
        "gnome-terminal",
        "xfce4-terminal",
        "konsole",
        "tilix",
        "mate-terminal",
        "lxterminal",
        "xterm",
        "x-terminal-emulator",
    ];
    for &t in CANDIDATES {
        if binary_exists(t) {
            return Some(t.to_string());
        }
    }
    if std::path::Path::new(paths::XTERM_BIN).exists() {
        return Some("xterm".to_string());
    }
    None
}

fn binary_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}
