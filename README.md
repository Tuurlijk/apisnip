# ApiSnip ✂️

A terminal user interface (TUI) tool for trimming OpenAPI specifications down to size. Apisnip allows you to interactively select which endpoints to keep in your API specification, making it easy to generate smaller, focused API surfaces.

![ApiSnip demo][apisnip.gif]

## ✨ Features

- 🖥️ Interactive TUI interface for selecting endpoints
- 📄 Support for both JSON and YAML OpenAPI specifications
- 🔗 Preserves all necessary references and components
- 🧩 Maintains original specification structure and order
- ⌨️ Keyboard and mouse navigation
- 🌈 Beautiful syntax highlighting for HTTP methods
- 🔍 Powerful fuzzy search with weighted scoring
- 🖱️ Click-to-select in the table view
- ⚡ Fast navigation with Page Up/Down and Home keys
- 🚀 Optimized for large API specifications
- 🌐 Support for remote API specifications via URLs
- 📊 Selected endpoints automatically grouped at the top
- 🌓 Automatic detection of system theme (light/dark mode)

## 🔧 Installation

```bash
cargo install apisnip
```

### Pre-built Binaries

Pre-built binaries and packages are available on the [GitHub releases](https://github.com/Tuurlijk/apisnip/releases) page for multiple platforms and architectures:

#### Binary Archives
- **Linux**: x86_64, ARM (32/64-bit), RISC-V, PowerPC, s390x, and MUSL variants
- **Windows**: 32-bit and 64-bit zip archives
- **macOS**: Intel and Apple Silicon (ARM64) builds
- **FreeBSD**: x86_64 builds

#### Package Formats
- **Debian/Ubuntu**: Native `.deb` packages
- **Red Hat/Fedora/SUSE**: RPM packages
- **Arch Linux**: AUR package
- **macOS**: Homebrew formula and DMG disk image
- **Nix**: Package for NixOS and Nix package manager

Each release includes SHA256 checksums for verifying file integrity.

#### Quick Installation

```bash
# Linux x86_64 example
curl -L https://github.com/Tuurlijk/apisnip/releases/download/[version]/apisnip-linux-x86_64.tar.gz | tar xz
./apisnip

# Or install using your system's package manager
# Debian/Ubuntu
sudo dpkg -i apisnip_[version]_amd64.deb

# Homebrew (macOS)
brew install apisnip
```

Replace `[version]` with the desired release version (e.g., `v1.4.56`).

## 📖 Usage

```bash
apisnip input [output.yaml]
```

### Arguments

- `input`: The input OpenAPI specification (required)
  - Can be a local file path (JSON or YAML)
  - Can be a URL to a remote specification (e.g., `https://example.com/api.yaml`)
- `output.yaml`: The output file path (optional, defaults to "apisnip.out.yaml")

### 🎮 Controls

- `↑` or `k`: Move selection up
- `↓` or `j`: Move selection down
- `Space`: Toggle selection of current endpoint ✂️
- `/`: Activate search mode 🔍
- `Esc`: Exit search mode
- `Page Up`: Scroll up one page
- `Page Down`: Scroll down one page
- `Home / End`: Jump to the top or bottom of the list 🔝
- `w`: Write selected endpoints to output file and quit
- `q`: Quit without saving
- Mouse scroll: Navigate through endpoints
- Mouse click: Select endpoint

## 🔍 Search Features

ApiSnip includes a powerful fuzzy search:

- Press `/` to enter search mode
- Type to filter endpoints by path and description
- Results are ranked with path matches weighted higher than description matches
- Selected items always appear at the top of results
- Best matches appear first within their selection group
- Selected items remain selected between searches
- Search is case-insensitive
- Press `Esc` to exit search and restore the full list

## 📋 User Interface

ApiSnip provides an intuitive interface for managing API endpoints:

- **Smart sorting**: Selected endpoints automatically move to the top of the list for better visibility
- **Context preservation**: When selecting items, the focus follows your natural workflow, avoiding disruptive jumps
- **Detailed view**: View comprehensive endpoint details in the bottom panel
- **Selection counter**: Track how many endpoints you've selected with the counter in the detail view
- **Adaptive theming**: Automatically detects your system's light/dark mode preference and adjusts colors to ensure optimal readability in any environment

## 📋 Examples

```bash
# Read from local file and write to output.yaml
apisnip input.yaml output.yaml

# Read from local JSON file and write to apisnip.out.yaml
apisnip input.json

# Read from remote URL and write to custom output file
apisnip https://petstore.swagger.io/v2/swagger.json my-petstore-api.yaml
```

## 🛠️ Development

### Building from Source

```bash
git clone https://github.com/Tuurlijk/apisnip.git
cd apisnip
cargo build --release
```

## 📝 Todo

- [ ] Enable user specified default styles or some sort of theming configuration
- [x] Replace linear color gradient with smooth sine wave gradient for more natural visual transitions

[apisnip.gif]: https://github.com/Tuurlijk/apisnip/blob/images/images/apisnip.gif?raw=true