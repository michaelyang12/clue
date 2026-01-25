# knock

knock is a CLI command helper written in Rust.

Simply describe what you want to do in plain English and knock returns the best matching shell command, automatically copied to your clipboard.

Example:

    kn "git clean commit history"

Output:

    git rebase -i --root

---

Installation

Requires Rust and Cargo.

    cargo install --git https://github.com/michaelyang12/knock.git --locked

This installs the `kn` binary into ~/.cargo/bin

---

Environment Setup

knock uses the OpenAI API and requires an API key.

1. Create an OpenAI account
You must have an OpenAI account with billing enabled.

2. Set your API key

    export OPENAI_API_KEY="your_api_key_here"

Add this to your shell profile (.bashrc, .zshrc, etc.) to persist it.

---

Usage

Basic usage:

    kn "find all large files over 500MB"

Verbose mode (recommended if you want to review options):

    kn --verbose "undo last git commit but keep changes"

or:

    kn -v "undo last git commit but keep changes"

Verbose mode returns:
- The top 5 command suggestions
- A brief explanation and example for each

Notes

- Generated commands are not executed automatically
- Always review commands before running them
- Use verbose mode if you are unsure about a result

---

License

MIT
