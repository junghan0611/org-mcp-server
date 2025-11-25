//! Test multi-silo functionality
//!
//! Run with: cargo run --example silo_test

use org_core::{OrgConfig, OrgMode};

fn main() {
    let config = OrgConfig {
        org_directory: "/home/goqual/org".to_string(),
        org_silo_roots: vec!["/home/goqual/repos/gh".to_string()],
        ..OrgConfig::default()
    };

    println!("=== Discovered Repo Docs ===");
    let discovered = config.discover_repo_docs();
    for d in &discovered {
        // Extract repo name from path
        let parts: Vec<&str> = d.split('/').collect();
        if parts.len() >= 2 {
            let repo_name = parts[parts.len() - 2];
            println!("  {:<25} -> {}", repo_name, d);
        }
    }
    println!("Total: {} repos with docs\n", discovered.len());

    // Now test with OrgMode - show files from repo docs only
    let org_mode = OrgMode::new(config).expect("Failed to create OrgMode");

    println!("=== Files from Repo Docs (excluding ~/org) ===");
    let files = org_mode
        .list_files_all_silos(None, None)
        .expect("Failed to list files");

    // Group by silo
    let mut by_silo: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
    for file in files {
        if !file.silo_name.starts_with("org") {
            by_silo.entry(file.silo_name.clone()).or_default().push(file);
        }
    }

    for (silo_name, files) in &by_silo {
        println!("\n[{}] ({} files)", silo_name, files.len());
        for file in files.iter().take(5) {
            let denote_info = if let Some(ref d) = file.denote {
                format!(" [{}]", d.identifier)
            } else {
                String::new()
            };
            println!("  - {}{}", file.relative_path, denote_info);
        }
        if files.len() > 5 {
            println!("  ... and {} more", files.len() - 5);
        }
    }

    println!("\n=== Summary ===");
    println!("Total silos with docs: {}", by_silo.len());
    let total_files: usize = by_silo.values().map(|v| v.len()).sum();
    println!("Total files in repo docs: {}", total_files);
}
