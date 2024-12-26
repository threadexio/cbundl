use eyre::Result;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Minify {
    remove_comments: bool,
    remove_whitespace: bool,
    shorten_identifiers: bool,
}

impl Minify {
    pub fn new(remove_comments: bool, remove_whitespace: bool, shorten_identifiers: bool) -> Self {
        Self {
            remove_comments,
            remove_whitespace,
            shorten_identifiers,
        }
    }

    pub fn minify(&self, code: &str) -> Result<String> {
        let mut minified_code = code.to_string();

        if self.remove_comments {
            minified_code = self.remove_comments(&minified_code);
        }

        if self.remove_whitespace {
            minified_code = self.remove_whitespace(&minified_code);
        }

        if self.shorten_identifiers {
            minified_code = self.shorten_identifiers(&minified_code);
        }

        Ok(minified_code)
    }

    fn remove_comments(&self, code: &str) -> String {
        // Regex for single-line comments (//...)
        let single_line_re = Regex::new(r"//[^\n]*").unwrap();
        let block_comment_re = Regex::new(r"/\*.*?\*/").unwrap();
        
        let code = single_line_re.replace_all(code, "");
        
        // Remove block comments while preserving newlines
        let code = block_comment_re.replace_all(&code, |caps: &regex::Captures| {
            let comment = &caps[0];
            // Replace block comments with newlines to keep the structure intact
            let lines = comment.lines().count();
            "\n".repeat(lines)
        });

        code.to_string()
    }

    fn remove_whitespace(&self, code: &str) -> String {
        // Split lines and trim each line 
        code.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty()) 
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn shorten_identifiers(&self, code: &str) -> String {
        let mut identifier_map: HashMap<String, String> = HashMap::new();
        let mut next_id = 0;

        // Closure to generate short id
        let mut generate_short_id = || {
            let id = format!("v{}", next_id);
            next_id += 1;
            id
        };

        // Regex for capturing variable declarations (including array/pointer types)
        let var_decl_re = Regex::new(r"\b(int|float|char|double|long|short|unsigned|signed|void)\s+([a-zA-Z_][a-zA-Z0-9_]*)(\s*\[\s*\d*\s*\])?(\s*\*+)?").unwrap();
        
        // Regex for function parameters and loop variables
        let func_param_re = Regex::new(r"(\w+)\s+([a-zA-Z_][a-zA0-9_]*)(\s*\[\s*\d*\s*\])?(\s*\*+)?").unwrap();
        
        // Regex for string literals
        let string_literal_re = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#).unwrap();
        
        let mut minified_code = String::new();
        
        // Process each line of the code
        for line in code.lines() {
            let mut modified_line = line.to_string();

            // Preserve #include lines
            if modified_line.starts_with("#include") {
                minified_code.push_str(&modified_line);
                minified_code.push('\n');
                continue;
            }

            // Skip string literals and comments for identifier shortening
            let mut non_string_code = modified_line.clone();
            let mut strings: Vec<String> = Vec::new();
            
            for cap in string_literal_re.captures_iter(&modified_line) {
                let string_literal = cap.get(0).unwrap().as_str().to_string();
                let placeholder = format!("__STRING_LITERAL_{}", strings.len());
                strings.push(string_literal.clone()); 
                non_string_code = non_string_code.replace(&string_literal, &placeholder);
            }

            // Skip shortening identifiers in 'main' function
            if !modified_line.contains("main") {
                // Handle variable declarations and function parameters
                non_string_code = var_decl_re.replace_all(&non_string_code, |caps: &regex::Captures| {
                    let original_name = caps[2].to_string();
                    let short_name = identifier_map.entry(original_name.clone()).or_insert_with(|| generate_short_id());
                    format!("{} {}", &caps[1], short_name)
                }).to_string();
                
                non_string_code = func_param_re.replace_all(&non_string_code, |caps: &regex::Captures| {
                    let original_name = caps[2].to_string();
                    let short_name = identifier_map.entry(original_name.clone()).or_insert_with(|| generate_short_id());
                    format!("{} {}", &caps[1], short_name)
                }).to_string();
            }

            for (original, short) in &identifier_map {
                let regex = Regex::new(&format!(r"\b{}\b", regex::escape(original))).unwrap();
                non_string_code = regex.replace_all(&non_string_code, short).to_string();
            }

            let mut final_line = non_string_code;
            for (i, placeholder) in strings.iter().enumerate() {
                final_line = final_line.replace(&format!("__STRING_LITERAL_{}", i), placeholder);
            }

            minified_code.push_str(&final_line);
            minified_code.push('\n');
        }

        minified_code
    }
}
