#!/bin/bash
# Test escape sequence handling

echo "Testing cursor movement and line erase..."
echo ""

# Draw 5 lines
echo "Line 1"
echo "Line 2"
echo "Line 3"
echo "Line 4"
echo "Line 5"

sleep 1

# Move up 3 lines and erase
printf '\033[3A'    # Move up 3 lines
printf '\033[2K'    # Erase entire line
printf 'REPLACED LINE 3\n'

sleep 1

# Move up 2 lines, erase, and write
printf '\033[2A'    # Move up 2 lines
printf '\033[2K'    # Erase entire line
printf 'REPLACED LINE 2\n'

sleep 1

echo ""
echo "If cursor movement works correctly:"
echo "- You should see REPLACED LINE 2 and REPLACED LINE 3"
echo "- Original Line 2 and Line 3 should be gone"
echo ""
echo "Press Enter to test Claude-style redraw..."
read

# Now test Claude's pattern: erase + move up repeatedly
echo "Testing Claude-style redraw pattern..."
echo ""
echo "Input: > test"
echo "─────────────"
echo ""

sleep 1

# Claude's pattern: [2K][1A repeated, then redraw
printf '\033[2K\033[1A'  # Erase and move up
printf '\033[2K\033[1A'  # Erase and move up
printf '\033[2K\033[1A'  # Erase and move up
printf '\033[G'          # Go to column 1

# Redraw
printf 'Input: > test_REDRAWN\n'
printf '─────────────────────\n'

echo ""
echo "Test complete. The 'Input' line should show 'test_REDRAWN'"
