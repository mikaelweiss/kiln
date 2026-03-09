use std::fs;
use std::path::Path;
use std::process::Command;

use dialoguer::{Input, Select};

use crate::platform::Platform;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let name: String = Input::new()
        .with_prompt("Project name")
        .validate_with(|input: &String| -> Result<(), &str> {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return Err("Project name cannot be empty");
            }
            if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains(' ') {
                return Err("Project name cannot contain spaces or path separators");
            }
            Ok(())
        })
        .interact_text()?;
    let name = name.trim().to_string();

    let platforms = Platform::ALL;
    let selection = Select::new()
        .with_prompt("Platform")
        .items(platforms)
        .default(0)
        .interact()?;
    let platform = platforms[selection];

    let project_path = Path::new(&name);
    if project_path.exists() {
        return Err(format!("Directory '{}' already exists", name).into());
    }

    fs::create_dir(&name)?;
    fs::create_dir(project_path.join("specs"))?;

    fs::write(
        project_path.join("SETUP_GUIDE.md"),
        format!(
            "# {} Setup Guide\n\n\
             Platform: {}\n\n\
             ## Checklist\n\n\
             - [ ] Define project specifications in `specs/`\n\
             - [ ] Configure code signing and build settings\n\
             - [ ] Set up CI/CD defaults\n\
             - [ ] Verify the project builds and runs\n",
            name, platform
        ),
    )?;

    fs::write(
        project_path.join("CLAUDE.md"),
        format!(
            "# {}\n\n\
             Platform: {}\n\n\
             <!-- Project-specific instructions for AI agents go here -->\n",
            name, platform
        ),
    )?;

    let git_result = Command::new("git")
        .args(["init"])
        .current_dir(&name)
        .output();

    match git_result {
        Ok(output) if output.status.success() => {}
        Ok(_) => eprintln!("Warning: git init failed. You can initialize the repository manually."),
        Err(_) => eprintln!("Warning: git not found. You can initialize the repository manually."),
    }

    println!("\nCreated project '{}' with platform {}", name, platform);
    println!("  {}/specs/", name);
    println!("  {}/SETUP_GUIDE.md", name);
    println!("  {}/CLAUDE.md", name);
    println!("\nOpen the project and point your AI agent at SETUP_GUIDE.md to continue setup.");

    Ok(())
}
