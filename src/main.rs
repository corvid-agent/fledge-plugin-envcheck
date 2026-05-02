#[link(wasm_import_module = "fledge")]
extern "C" {
    fn recv(ptr: *mut u8, max_len: i32) -> i32;
    fn send(ptr: *const u8, len: i32);
    fn exit(code: i32);
}

fn fledge_recv() -> Vec<u8> {
    let mut buf = vec![0u8; 65536];
    let len = unsafe { recv(buf.as_mut_ptr(), buf.len() as i32) };
    buf.truncate(len.max(0) as usize);
    buf
}

fn fledge_send(msg: &str) {
    unsafe { send(msg.as_ptr(), msg.len() as i32) };
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

fn output(text: &str) {
    fledge_send(&format!(r#"{{"type":"output","text":"{}"}}"#, json_escape(text)));
}

fn parse_env_keys(content: &str) -> Vec<(String, bool)> {
    let mut keys = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            if !key.is_empty() {
                let value = trimmed[eq_pos + 1..].trim();
                let has_value = !value.is_empty() && value != "\"\"" && value != "''";
                keys.push((key, has_value));
            }
        }
    }
    keys
}

fn find_env_files(path: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return pairs,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".env.example" || name == ".env.sample" || name == ".env.template" {
            let example_path = format!("{}/{}", path, name);
            let env_path = format!("{}/.env", path);
            pairs.push((example_path, env_path));
        }
    }

    // Also recurse into subdirectories (but not too deep)
    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return pairs,
    };

    for entry in entries.flatten() {
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        if ft.is_dir() && !name.starts_with('.')
            && name != "node_modules" && name != "target" && name != "vendor"
        {
            let sub = format!("{}/{}", path, name);
            pairs.extend(find_env_files(&sub));
        }
    }

    pairs
}

fn main() {
    let _init = fledge_recv();

    let pairs = find_env_files("/project");

    if pairs.is_empty() {
        output("\n  No .env.example/.env.sample/.env.template files found.\n");
        output("  Nothing to check.\n\n");
        unsafe { exit(0) };
        unreachable!();
    }

    let mut total_missing = 0u32;
    let mut total_extra = 0u32;
    let mut total_empty = 0u32;

    for (example_path, env_path) in &pairs {
        let display_dir = if example_path.starts_with("/project/") {
            let trimmed = &example_path[9..];
            if let Some(slash) = trimmed.rfind('/') {
                &trimmed[..slash]
            } else {
                "."
            }
        } else {
            example_path.as_str()
        };
        let example_name = example_path.rsplit('/').next().unwrap_or("example");

        output(&format!("\n  {} ({})\n", display_dir, example_name));
        output("  ────────────────────────────────────────\n");

        let example_content = match std::fs::read_to_string(example_path) {
            Ok(c) => c,
            Err(e) => {
                output(&format!("  \u{2717} Cannot read {}: {}\n", example_name, e));
                continue;
            }
        };

        let example_keys = parse_env_keys(&example_content);

        let env_content = match std::fs::read_to_string(env_path) {
            Err(_) => {
                output(&format!("  \u{2717} .env file missing! {} required keys:\n", example_keys.len()));
                for (key, _) in &example_keys {
                    output(&format!("    - {}\n", key));
                    total_missing += 1;
                }
                continue;
            }
            Ok(c) => c,
        };

        let env_keys = parse_env_keys(&env_content);
        let env_key_names: Vec<&str> = env_keys.iter().map(|(k, _)| k.as_str()).collect();
        let example_key_names: Vec<&str> = example_keys.iter().map(|(k, _)| k.as_str()).collect();

        // Find missing keys (in example but not in .env)
        let mut missing = Vec::new();
        for key in &example_key_names {
            if !env_key_names.contains(key) {
                missing.push(*key);
            }
        }

        // Find extra keys (in .env but not in example)
        let mut extra = Vec::new();
        for key in &env_key_names {
            if !example_key_names.contains(key) {
                extra.push(*key);
            }
        }

        // Find empty values (key exists but no value set)
        let mut empty = Vec::new();
        for (key, has_value) in &env_keys {
            if example_key_names.contains(&key.as_str()) && !has_value {
                empty.push(key.as_str());
            }
        }

        if missing.is_empty() && extra.is_empty() && empty.is_empty() {
            output(&format!("  \u{2713} All {} keys present and set\n", example_keys.len()));
        } else {
            if !missing.is_empty() {
                output(&format!("  \u{2717} Missing ({}):\n", missing.len()));
                for key in &missing {
                    output(&format!("    - {}\n", key));
                }
                total_missing += missing.len() as u32;
            }

            if !empty.is_empty() {
                output(&format!("  \u{26a0} Empty ({}):\n", empty.len()));
                for key in &empty {
                    output(&format!("    - {}\n", key));
                }
                total_empty += empty.len() as u32;
            }

            if !extra.is_empty() {
                output(&format!("  \u{2139} Extra ({}):\n", extra.len()));
                for key in &extra {
                    output(&format!("    + {}\n", key));
                }
                total_extra += extra.len() as u32;
            }
        }
    }

    output("\n  ────────────────────────────────────────\n");
    if total_missing == 0 && total_empty == 0 {
        output("  \u{2713} All environment files are complete.\n\n");
    } else {
        if total_missing > 0 {
            output(&format!("  {} missing key(s) — add to .env\n", total_missing));
        }
        if total_empty > 0 {
            output(&format!("  {} empty value(s) — set in .env\n", total_empty));
        }
        if total_extra > 0 {
            output(&format!("  {} extra key(s) — consider adding to example\n", total_extra));
        }
        output("\n");
    }

    unsafe { exit(0) };
    unreachable!();
}
