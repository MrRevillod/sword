use regex_lite::Regex;
use std::env;

pub fn expand_env_variables(content: &str) -> Result<String, String> {
    let re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*):?([^}]*)\}")
        .map_err(|e| format!("Regex error: {e}"))?;

    let mut result = content.to_string();

    for caps in re.captures_iter(content) {
        let full_match = caps.get(0).unwrap().as_str();
        let var_name = caps.get(1).unwrap().as_str();
        let default_value = caps.get(2).map_or("", |m| m.as_str());

        let replacement = match env::var(var_name) {
            Ok(value) => value,
            Err(_) => {
                if default_value.is_empty() {
                    return Err(format!(
                        "environment variable '{var_name}' not found"
                    ));
                } else {
                    default_value.to_string()
                }
            }
        };

        result = result.replace(full_match, &replacement);
    }

    let simple_re = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)")
        .map_err(|e| format!("Regex error: {e}"))?;

    for caps in simple_re.captures_iter(&result.clone()) {
        let full_match = caps.get(0).unwrap().as_str();
        let var_name = caps.get(1).unwrap().as_str();

        if content.contains(&format!("${{{var_name}")) {
            continue;
        }

        let Ok(replacement) = env::var(var_name) else {
            return Err(format!("environment variable '{var_name}' not found"));
        };

        result = result.replace(full_match, &replacement);
    }

    Ok(result)
}
