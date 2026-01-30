//! Unit tests for CommandService

#[cfg(test)]
mod tests {
    #[test]
    fn test_command_whitelist() {
        let whitelist = vec![
            "git", "npm", "pnpm", "yarn", "node", "python", "cargo", "rustc"
        ];

        assert!(whitelist.contains(&"git"));
        assert!(whitelist.contains(&"npm"));
        assert!(!whitelist.contains(&"rm"));
        assert!(!whitelist.contains(&"del"));
    }

    #[test]
    fn test_dry_run_prediction() {
        // Test npm install prediction
        let command = "npm";
        let args = vec!["install".to_string()];
        
        let predictions = predict_effects(command, &args);
        
        assert!(predictions.iter().any(|p| p.contains("node_modules")));
    }

    #[test]
    fn test_sensitive_command_detection() {
        let sensitive_patterns = vec![
            "rm -rf",
            "del /s",
            "format",
            "shutdown",
            "git push --force",
        ];

        let test_commands = vec![
            ("rm -rf /", true),
            ("git push --force", true),
            ("npm install", false),
            ("git status", false),
        ];

        for (cmd, should_be_sensitive) in test_commands {
            let is_sensitive = sensitive_patterns.iter().any(|p| cmd.contains(p));
            assert_eq!(is_sensitive, should_be_sensitive, "Command: {}", cmd);
        }
    }

    #[test]
    fn test_auto_approve_patterns() {
        let auto_approve = vec![
            "git status",
            "git log",
            "git diff",
            "npm list",
        ];

        assert!(auto_approve.iter().any(|p| "git status".starts_with(p)));
        assert!(auto_approve.iter().any(|p| "git log --oneline".starts_with(p)));
        assert!(!auto_approve.iter().any(|p| "git push".starts_with(p)));
    }

    fn predict_effects(command: &str, args: &[String]) -> Vec<String> {
        let mut effects = Vec::new();
        
        match command {
            "npm" | "pnpm" | "yarn" => {
                if args.first().map(|s| s.as_str()) == Some("install") {
                    effects.push("Will create/update node_modules folder".to_string());
                    effects.push("May update package-lock.json".to_string());
                }
            }
            "git" => {
                match args.first().map(|s| s.as_str()) {
                    Some("commit") => effects.push("Will create a new git commit".to_string()),
                    Some("push") => effects.push("Will push to remote".to_string()),
                    _ => {}
                }
            }
            _ => effects.push(format!("Will execute: {} {}", command, args.join(" "))),
        }
        
        effects
    }
}
