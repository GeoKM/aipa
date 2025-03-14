# AIPA - AI Programming Agent

**Version: 0.2.0**

AIPA (AI Programming Agent) is a command-line tool that automates code generation, compilation, execution, and error correction for multiple programming languages. It’s designed to streamline prototyping and debugging by generating initial code based on a goal, running it, and allowing iterative fixes when errors occur.

## Features
- **Supported Languages**: Rust, Python, C++, Java
- **Code Generation**: Creates simple programs based on a user-defined goal (e.g., "print hello").
- **Error Iteration**: Detects compilation errors, prompts for fixes, and retries up to 3 times.
- **File Cleanup**: Automatically removes generated source and binary files after completion.
- **Debug Mode**: Verbose output for troubleshooting with `--debug`.

## Installation
1. **Prerequisites**:
   - Rust (with `cargo`) for building AIPA.
   - Compilers: `rustc` (Rust), `python` (Python), `g++` (C++), `javac` and `java` (Java).
   - On macOS, ensure Xcode Command Line Tools are installed (`xcode-select --install`).

2. **Clone the Repo**:
   ```bash
   git clone https://github.com/GeoKM/aipa.git
   cd aipa
   ```
    Build:
    ```bash

    cargo build --release
    ```
    Run:
        Use the release binary: ./target/release/aipa
        Or run directly with Cargo: cargo run --

Usage

Run AIPA with a language and goal:
```bash
cargo run -- --language <lang> --goal "<goal>" [--debug]
```
    <lang>: rust, python, cpp, or java.
    <goal>: A short description (e.g., "print hello").
    --debug: Optional, enables verbose output.

Example

Generate and run a "print hello" program in Java:
```bash
cargo run -- --language java --goal "print hello" --debug
```
    If an error occurs, AIPA prompts for a fix (enter code, press Enter twice to submit).

Error Iteration

    AIPA detects compile errors and allows up to 3 attempts to fix them.
    Example input for a fix (type or paste code, then Enter twice):
    text

    > public class project_print_hello {
    >     public static void main(String[] args) {
    >         System.out.println("AIPA: print hello completed");
    >     }
    > }
    >

Project Status

    Version 0.2.0: Stable with basic language support, error iteration, and cleanup.
    In Development: Future enhancements may include runtime error handling, output polishing, and more languages.

Contributing

Feel free to fork, submit issues, or send pull requests! Key areas for improvement:

    Additional language support (e.g., Go, C#).
    Runtime error detection and iteration.
    User interface enhancements.

License

This project is unlicensed—free to use, modify, and distribute as you see fit.
Acknowledgments

Built with help from Grok (xAI) for rapid prototyping and debugging.
