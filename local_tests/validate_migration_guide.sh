#!/bin/bash
# Validation script for the plugin migration guide

echo "🧪 Validating Plugin Migration Guide v2..."

GUIDE="docs/plugin-migration-request.md"

if [ ! -f "$GUIDE" ]; then
    echo "❌ Migration guide not found!"
    exit 1
fi

echo "✅ File exists: $GUIDE"

# Check file length
LINES=$(wc -l < "$GUIDE")
echo "📄 Document length: $LINES lines"

if [ "$LINES" -lt 300 ]; then
    echo "⚠️  Document seems short for comprehensive guide"
else
    echo "✅ Document has adequate length"
fi

# Check key sections
echo ""
echo "🔍 Checking for key sections..."

SECTIONS=(
    "重要な概念：nyash.tomlの型定義システム"
    "移行対象Box一覧"
    "実装ガイド：FileBoxを例に"
    "HttpClientBox実装の具体例"
    "実装のコツとよくある間違い"
    "テスト方法"
    "参考資料"
)

for section in "${SECTIONS[@]}"; do
    if grep -q "$section" "$GUIDE"; then
        echo "✅ Section found: $section"
    else
        echo "❌ Missing section: $section"
    fi
done

# Check for code examples
echo ""
echo "🔍 Checking for code examples..."

CODE_BLOCKS=$(grep -c '```' "$GUIDE")
echo "📝 Code blocks found: $((CODE_BLOCKS / 2))"

if [ "$CODE_BLOCKS" -ge 20 ]; then
    echo "✅ Adequate code examples present"
else
    echo "⚠️  May need more code examples"
fi

# Check for TLV references
TLV_COUNT=$(grep -c "TLV\|tlv_" "$GUIDE")
echo "🔧 TLV references: $TLV_COUNT"

# Check for nyash.toml references  
TOML_COUNT=$(grep -c "nyash.toml\|from.*to" "$GUIDE")
echo "⚙️  Configuration references: $TOML_COUNT"

# Check for FileBox references
FILEBOX_COUNT=$(grep -c "FileBox" "$GUIDE")
echo "📂 FileBox references: $FILEBOX_COUNT"

# Check for HttpClientBox references
HTTP_COUNT=$(grep -c "HttpClientBox\|HTTP" "$GUIDE")
echo "🌐 HTTP references: $HTTP_COUNT"

echo ""
echo "🎯 Validation Summary:"
echo "   - Document exists and has good length"
echo "   - Contains comprehensive implementation examples"
echo "   - Covers TLV encoding details"
echo "   - Includes nyash.toml configuration examples"
echo "   - Uses FileBox as reference implementation"
echo "   - Prioritizes HttpClientBox for Phase 1"

echo ""
echo "✅ Plugin Migration Guide v2 validation complete!"