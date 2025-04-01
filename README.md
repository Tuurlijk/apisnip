# ApiSnip

A terminal user interface (TUI) tool for trimming OpenAPI specifications down to size. Apisnip allows you to interactively select which endpoints to keep in your API specification, making it easy to generate smaller, focused API surfaces.

![ApiSnip demo][apisnip.gif]

## Features

- Interactive TUI interface for selecting endpoints
- Support for both JSON and YAML OpenAPI specifications
- Preserves all necessary references and components
- Maintains original specification structure and order
- Keyboard and mouse navigation
- Beautiful syntax highlighting for HTTP methods

## Installation

```bash
cargo install apisnip
```

## Usage

```bash
apisnip input.yaml [output.yaml]
```

### Arguments

- `input.yaml`: The input OpenAPI specification file (required)
- `output.yaml`: The output file path (optional, defaults to "apisnip.out.yaml")

### Controls

- `↑` or `k`: Move selection up
- `↓` or `j`: Move selection down
- `Space`: Toggle selection of current endpoint
- `w`: Write selected endpoints to output file and quit
- `q`: Quit without saving
- Mouse scroll: Navigate through endpoints
- Mouse click: Select endpoint

## Example

```bash
# Read from input.yaml and write to output.yaml
apisnip input.yaml output.yaml

# Read from input.json and write to apisnip.out.yaml
apisnip input.json
```

## Development

### Building from Source

```bash
git clone https://github.com/michielroos/apisnip.git
cd apisnip
cargo build --release
```

[apisnip.gif]: https://github.com/Tuurlijk/apisnip/blob/images/images/apisnip.gif?raw=true