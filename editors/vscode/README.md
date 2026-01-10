# Nostos for VS Code

Syntax highlighting and language support for the [Nostos programming language](https://github.com/pegesund/nostos).

## Features

- Syntax highlighting for `.nos` files
- Bracket matching and auto-closing
- Comment toggling (`Ctrl+/` or `Cmd+/`)
- Code folding

## Installation

### From VSIX (Local Install)

1. Download the `.vsix` file from releases
2. In VS Code: `Extensions` → `...` → `Install from VSIX...`
3. Select the downloaded file

### From Source

```bash
cd editors/vscode
npm install
npm run package
# This creates nostos-0.1.0.vsix
```

Then install the generated `.vsix` file.

## Language Overview

Nostos is a functional programming language with:

- Pattern matching
- Type inference
- Async/concurrent primitives (`spawn`, `mvar`)
- Trait system
- ML-style syntax

## Example

```nostos
# HTTP server example
use stdlib.server.{serve, respondHtml}

handleRequest(req) = match req.path {
    "/" -> respondHtml(req, "<h1>Hello, World!</h1>")
    "/api" -> respondJson(req, {status: "ok"})
    _ -> respond404(req)
}

main() = {
    println("Server starting on port 8080")
    serve(8080, handleRequest)
}
```

## Contributing

Issues and pull requests welcome at [github.com/pegesund/nostos](https://github.com/pegesund/nostos).

## License

MIT
