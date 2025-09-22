// n34 - A CLI to interact with NIP-34 and other stuff related to codes in nostr
// Copyright (C) 2025 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://gnu.org/licenses/gpl-3.0.html>.

use std::{
    fs,
    io::{self, Write},
};

use crate::error::{N34Error, N34Result};

/// Displays the given prompt and reads a line of input from the user.
pub fn read_line(prompt: &str) -> io::Result<String> {
    {
        let mut stdout = io::stdout().lock();

        write!(&mut stdout, "{prompt}: ")?;
        _ = stdout.flush();
    }
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;
    Ok(user_input.trim().to_owned())
}

/// Prompts the user with a message and repeatedly asks until they enter a valid
/// boolean response. Recognizes "yes", "y", "true" for `true` and "no", "n",
/// "false" for `false`.
pub fn prompt_bool(prompt: &str) -> io::Result<bool> {
    loop {
        let user_input = read_line(prompt)?.to_ascii_lowercase();

        match user_input.as_str() {
            "yes" | "y" | "true" => return Ok(true),
            "no" | "n" | "false" => return Ok(false),
            _ => continue,
        }
    }
}

/// Opens the logs file for writing. If the file size exceeds 5MB, it is opened
/// in write mode, otherwise in append mode.
pub fn logs_file() -> N34Result<fs::File> {
    const FIVE_MB: u64 = 1024 * 1024 * 5;

    let logs_path = dirs::data_local_dir()
        .ok_or(N34Error::CanNotFindDataPath)?
        .join("n34")
        .join("logs.log");

    tracing::info!(path = %logs_path.display(), "Logs file");

    if let Some(parent) = logs_path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }

    _ = fs::File::create_new(&logs_path);

    let is_large = if let Ok(file) = fs::File::open(&logs_path)
        && let Ok(metadata) = file.metadata()
    {
        metadata.len() >= FIVE_MB
    } else {
        false
    };

    fs::OpenOptions::new()
        .write(true)
        .append(!is_large)
        .truncate(is_large)
        .open(&logs_path)
        .map_err(N34Error::from)
}
