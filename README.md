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
- ⚡ Fast navigation with Page Up/Down
- 🚀 Optimized for large API specifications
- 🌐 Support for remote API specifications via URLs

## 🔧 Installation

```bash
cargo install apisnip
```

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
- `w`: Write selected endpoints to output file and quit
- `q`: Quit without saving
- Mouse scroll: Navigate through endpoints
- Mouse click: Select endpoint

## 🔍 Search Features

ApiSnip includes a powerful fuzzy search:

- Press `/` to enter search mode
- Type to filter endpoints by path and description
- Results are ranked with path matches weighted higher than description matches
- Best matches appear at the top
- Selected items remain selected between searches
- Search is case-insensitive
- Press `Esc` to exit search and restore the full list

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
git clone https://github.com/michielroos/apisnip.git
cd apisnip
cargo build --release
```

[apisnip.gif]: https://github.com/Tuurlijk/apisnip/blob/images/images/apisnip.gif?raw=true