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

Pre-built binary packages are available in the [GitHub releases](https://github.com/Tuurlijk/apisnip/releases). Each release includes a variety of package formats for different platforms:

- **Linux**: `.deb`, `.rpm`, `.AppImage` (x86_64 only), and `.tar.gz` archives
- **macOS**: `.dmg` and `.tar.gz` archives
- **Windows**: `.msi` installers and `.zip` archives

We support a wide range of architectures:
- x86_64 (64-bit Intel/AMD)
- aarch64/arm64 (64-bit ARM)
- armv7 (32-bit ARM v7)
- arm (32-bit ARM)
- i686/i386 (32-bit Intel/AMD)
- RISC-V 64-bit
- x86_64 with MUSL libc

To download and use a pre-built binary:

1. Visit the [latest release page](https://github.com/Tuurlijk/apisnip/releases/latest)
2. Download the appropriate package for your platform
3. Install using your platform's standard method:
   - Linux: Use your package manager with `.deb`/`.rpm` or run the `.AppImage`
   - macOS: Open the `.dmg` and drag to Applications
   - Windows: Run the `.msi` installer

If you prefer not to use installers, the `.tar.gz` and `.zip` archives contain standalone binaries.

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