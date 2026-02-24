use std::io::{self, Write};

use anyhow::Result;

/// Prompt the user for input on stdin. Prints `label: ` and reads one line.
pub fn prompt(label: &str) -> Result<String> {
    print!("{label}: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

/// Prompt with a default value shown in brackets.
pub fn prompt_default(label: &str, default: &str) -> Result<String> {
    let input = prompt(&format!("{label} [{default}]"))?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

/// Prompt for yes/no confirmation.
pub fn confirm(label: &str, default_yes: bool) -> Result<bool> {
    let hint = if default_yes { "Y/n" } else { "y/N" };
    let input = prompt(&format!("{label} [{hint}]"))?;
    let trimmed = input.trim().to_lowercase();
    Ok(match trimmed.as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        "" => default_yes,
        _ => default_yes,
    })
}
