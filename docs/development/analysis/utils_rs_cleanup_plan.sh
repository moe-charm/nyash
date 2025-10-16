#!/bin/bash
# Phase 1 P1-P3 Docs - utils.rs cleanup script
# 目的: #[allow(dead_code)] マーカー8個削除（すべて実際には使用中）

set -e

TARGET_FILE="src/mir/builder/utils.rs"
BACKUP_FILE="$TARGET_FILE.backup"

echo "=== utils.rs Cleanup Script ==="
echo ""
echo "Target: $TARGET_FILE"
echo "Action: Remove 8 #[allow(dead_code)] markers"
echo ""

# Backup
echo "1. Creating backup..."
cp "$TARGET_FILE" "$BACKUP_FILE"
echo "   ✅ Backup created: $BACKUP_FILE"
echo ""

# Show current markers
echo "2. Current #[allow(dead_code)] markers:"
grep -n "#\[allow(dead_code)\]" "$TARGET_FILE" || echo "   (none found)"
echo ""

# Remove markers
echo "3. Removing markers..."
sed -i '/^    #\[allow(dead_code)\]$/d' "$TARGET_FILE"
echo "   ✅ Markers removed"
echo ""

# Verify
echo "4. Verification:"
remaining=$(grep -c "#\[allow(dead_code)\]" "$TARGET_FILE" || echo "0")
if [ "$remaining" -eq 0 ]; then
    echo "   ✅ All markers removed successfully"
else
    echo "   ⚠️ Warning: $remaining markers remaining"
fi
echo ""

# Check compilation
echo "5. Checking compilation..."
if cargo check --quiet 2>&1 | grep -i "dead_code.*$TARGET_FILE"; then
    echo "   ❌ Dead code warnings detected!"
    echo "   Restoring backup..."
    cp "$BACKUP_FILE" "$TARGET_FILE"
    exit 1
else
    echo "   ✅ No dead_code warnings"
fi
echo ""

# Show diff
echo "6. Changes made:"
diff -u "$BACKUP_FILE" "$TARGET_FILE" || true
echo ""

# Summary
old_lines=$(wc -l < "$BACKUP_FILE")
new_lines=$(wc -l < "$TARGET_FILE")
deleted=$((old_lines - new_lines))

echo "=== Summary ==="
echo "  Old lines: $old_lines"
echo "  New lines: $new_lines"
echo "  Deleted:   $deleted"
echo ""
echo "✅ Cleanup complete!"
echo ""
echo "Next steps:"
echo "  1. Run tests: tools/smokes/v2/run.sh --profile quick"
echo "  2. If all pass, commit: git add $TARGET_FILE"
echo "  3. Remove backup: rm $BACKUP_FILE"
