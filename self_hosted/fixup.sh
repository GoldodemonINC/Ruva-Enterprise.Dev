#!/bin/bash
# Post-process transpiled .rs files to fix known issues
FILE="$1"
# 1. Add type alias for string
sed -i '1a\type string = String;' "$FILE"
# 2. Fix double-escaped backslashes (transpiler bug)
sed -i 's/\\x1b/\x1b/g' "$FILE"
sed -i 's/\\n/\n/g' "$FILE"
sed -i 's/\\t/\t/g' "$FILE"
sed -i 's/\\r/\r/g' "$FILE"
sed -i 's/\\0/\0/g' "$FILE"
# 3. Remove inner allow attributes
sed -i '/#!\[allow/d' "$FILE"
echo "Fixed: $FILE"
