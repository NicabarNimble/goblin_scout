# Turn Back Now...

![Goblin Scout](src/assets/scout.png)

## About

Goblin Scout is a Git repository documentation tool that generates structured markdown files from your codebase. It's designed for creating both human-readable documentation and machine-processable datasets.

### Features

- **Repository Processing**:
  - Automatic cloning of new repositories
  - Smart updating of existing repositories
  - Git metadata extraction (contributors, releases)
  - Requires repositories to have at least one tag for release information

- **Documentation Modes**:
  1. Single File Mode
     - Combines all code into one markdown file
     - Preserves file structure in headers
     - Ideal for quick codebase overview

  2. Multi-File Mode
     - Creates separate markdown files for each source file
     - Maintains original directory structure
     - Each file includes complete metadata

  3. Dataset Mode
     - Splits code into manageable chunks (≤750 characters)
     - Adds unique UUIDs to each code section
     - Optional JSON conversion for dataset creation
     - Preserves code structure with smart chunk boundaries

- **Metadata Tracking**:
  - File statistics and paths
  - Contributor analysis with commit counts
  - Latest release version and date
  - Direct GitHub file URLs
  - Language detection
  - UUID tracking for files and code sections

## Usage

### Installation
```bash
# Clone and build the project
git clone https://github.com/yourusername/goblin_scout.git
cd goblin_scout
cargo build --release
```

### Basic Usage
```bash
cargo run --release
```

The tool will prompt you for:
1. Git repository URL
2. Output path for generated documentation

Then select your desired output format:
1. Single markdown file
2. Multiple markdown files (one per source file)
3. Dataset format with optional JSON conversion

### Requirements
- Target repository must have at least one Git tag
- Valid Git repository URL
- Write permissions for output directory

### Output Format

Generated files include:
- YAML frontmatter with metadata
- Code sections with syntax highlighting
- UUIDs for tracking (in dataset mode)
- Optional JSON conversion for dataset mode
