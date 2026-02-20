# API Examples

This directory contains example scripts demonstrating API usage.

## api_examples.sh

A bash script that demonstrates common API operations:
- Creating cards and decks
- Listing resources
- Combat initialization
- Validation examples

### Prerequisites

- Server running on http://localhost:8000
- `jq` installed for JSON formatting (optional but recommended)
- `curl` installed

### Usage

```bash
# Start the server in one terminal
cargo run

# In another terminal, run the examples
./examples/api_examples.sh
```

You can also run individual curl commands from the script manually to test specific endpoints.
