# kiln

An opinionated project scaffolding CLI. Pick a platform, name your project, and get a fully configured repository — ready for development, collaboration, and publishing.

Kiln is designed to work hand-in-hand with AI coding agents. It generates a project with a setup checklist that any agent (Claude Code, OpenCode, Gemini, Codex, etc.) can follow step by step to finish bootstrapping the project — writing specs, configuring signing, and committing each stage as it goes.

## How It Works

### 1. Create the project

```
kiln create
```

The CLI walks you through:

- **Project name** — propagated through all files, configs, and identifiers.
- **Platform** — iOS, macOS, Android, Web, Tauri, or Chrome Extension, CLI.

Kiln clones the matching template, renames everything to your project name, and initializes a git repository.

### 2. Hand off to an AI agent

Open the new repository in your AI coding tool of choice and point it at the generated setup guide. The guide contains an ordered checklist of setup tasks. The agent works through each item — defining specs, wiring up configuration, and making a git commit for every completed step.

### 3. Start building

When the checklist is done you have:

- A clean git history with one commit per setup step.
- A `specs/` folder with project specifications in a pre-defined format.
- Code signing, build configs, and CI defaults already in place.
- A repository ready for multiple developers from day one.

## Supported Platforms

| Platform         | Template |
|------------------|----------|
| iOS (Swift)      | Planned  |
| macOS (Swift)    | Planned  |
| Android (Kotlin) | Planned  |
| Web              | Planned  |
| Tauri            | Planned  |
| Chrome Extension | Planned  |
| CLI              | Planned  |

## Design Principles

- **Opinionated defaults.** Every template bakes in decisions about architecture, UI patterns, project structure, and tooling so you skip the boilerplate debate.
- **AI-agent friendly.** The setup guide is written as clear, sequential instructions that any AI coding agent can execute autonomously.
- **Publish-ready from the start.** Code signing, build configuration, and distribution scaffolding are part of the template — not an afterthought.
- **One commit per step.** The AI-driven setup produces a clean, reviewable git history where each commit corresponds to a single setup task.

## Project Structure (generated)

```
your-project/
├── specs/            # Project specifications in a standardized format
├── SETUP_GUIDE.md    # Ordered checklist for AI-assisted setup
├── ...               # Platform-specific source code and config
```

## License

