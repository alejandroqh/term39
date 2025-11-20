#!/bin/bash
# Capture raw PTY output from claude CLI

OUTPUT_FILE="claude_raw_output.bin"

echo "Recording claude output to $OUTPUT_FILE"
echo "Type some commands, then Ctrl+D or 'exit' to finish"
echo "---"

# Record raw terminal session
script -q "$OUTPUT_FILE" /Users/aquintanar/.local/bin/claude

echo "---"
echo "Raw output saved to $OUTPUT_FILE"
echo ""
echo "Analyzing output..."
echo ""

# Show hex dump of first 2000 bytes
echo "=== HEX DUMP (first 2000 bytes) ==="
xxd "$OUTPUT_FILE" | head -125

echo ""
echo "=== ESCAPE SEQUENCES FOUND ==="
# Find escape sequences (ESC = 0x1b)
grep -o $'\x1b\[[^m]*m\|\x1b\[[0-9;]*[A-Za-z]' "$OUTPUT_FILE" | sort | uniq -c | sort -rn | head -30

echo ""
echo "=== CARRIAGE RETURNS AND NEWLINES ==="
echo "CR (\\r) count: $(grep -c $'\r' "$OUTPUT_FILE" || echo 0)"
echo "LF (\\n) count: $(grep -c $'\n' "$OUTPUT_FILE" || echo 0)"
echo "CRLF (\\r\\n) count: $(grep -c $'\r\n' "$OUTPUT_FILE" || echo 0)"

echo ""
echo "Full hex dump: xxd $OUTPUT_FILE | less"
