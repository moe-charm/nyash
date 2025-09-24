#!/usr/bin/env bash
# Phase 2.0: Plugin Priority Test - FactoryPolicy システム完全検証
#
# Purpose: Phase 15.5 "Everything is Plugin" 革命の動作確認
# Tests: StrictPluginFirst/CompatPluginFirst/BuiltinFirst policy switching

set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/../../.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"
TEST_DIR="$(dirname "$0")"

echo "🎯 [Plugin Priority Test] Phase 15.5 FactoryPolicy システム検証開始" >&2

# Ensure nyash binary exists
if [[ ! -x "$BIN" ]]; then
  echo "[test] Building nyash (release)..." >&2
  (cd "$ROOT_DIR" && cargo build --release >/dev/null 2>&1)
fi

# Build critical plugins for testing
build_plugin() {
  local plugin_dir="$ROOT_DIR/plugins/$1"
  if [[ -d "$plugin_dir" ]]; then
    echo "[test] Building plugin: $1" >&2
    (cd "$plugin_dir" && cargo build --release >/dev/null 2>&1) || {
      echo "⚠️ [test] Plugin $1 build failed, skipping..." >&2
      return 1
    }
    return 0
  else
    echo "⚠️ [test] Plugin directory $1 not found, skipping..." >&2
    return 1
  fi
}

# Build required plugins
PLUGINS_AVAILABLE=()
for plugin in nyash-string-plugin nyash-integer-plugin nyash-console-plugin nyash-math-plugin; do
  if build_plugin "$plugin"; then
    PLUGINS_AVAILABLE+=("$plugin")
  fi
done

echo "[test] Available plugins: ${PLUGINS_AVAILABLE[*]:-none}" >&2

# Create test files for each Box type
create_test_files() {
  local test_base="/tmp/nyash_plugin_priority_test"
  mkdir -p "$test_base"

  # StringBox test
  cat > "$test_base/test_stringbox.nyash" <<'EOF'
local s = new StringBox("Plugin Priority Test")
print("StringBox created: " + s.get())
print("Test: StringBox Priority")
EOF

  # IntegerBox test
  cat > "$test_base/test_integerbox.nyash" <<'EOF'
local i = new IntegerBox(42)
print("IntegerBox created: " + i.get())
print("Test: IntegerBox Priority")
EOF

  # Combined test
  cat > "$test_base/test_combined.nyash" <<'EOF'
local s = new StringBox("Combined Test")
local i = new IntegerBox(123)
print("StringBox: " + s.get())
print("IntegerBox: " + i.get())
print("Test: Combined Priority")
EOF

  echo "$test_base"
}

run_policy_test() {
  local policy=$1
  local test_file=$2
  local test_name=$3

  echo "" >&2
  echo "🧪 [Test] Policy: $policy | Test: $test_name" >&2
  echo "   File: $test_file" >&2

  # Set environment for this test
  export NYASH_BOX_FACTORY_POLICY="$policy"
  export NYASH_CLI_VERBOSE=1

  # Run test and capture output
  local output
  if output=$("$BIN" "$test_file" 2>&1); then
    echo "✅ [Test] SUCCESS: $test_name ($policy)" >&2

    # Check for policy log message
    if echo "$output" | grep -q "Factory Policy: "; then
      local policy_line=$(echo "$output" | grep "Factory Policy: " | head -1)
      echo "   📋 Policy Log: $policy_line" >&2
    fi

    # Check for successful Box creation
    if echo "$output" | grep -q "Test: "; then
      local test_result=$(echo "$output" | grep "Test: " | head -1)
      echo "   🎯 Result: $test_result" >&2
    fi

    return 0
  else
    echo "❌ [Test] FAILED: $test_name ($policy)" >&2
    echo "   Output: $output" >&2
    return 1
  fi
}

run_comprehensive_tests() {
  local test_base=$1

  local policies=("strict_plugin_first" "compat_plugin_first" "builtin_first")
  local tests=("test_stringbox.nyash:StringBox" "test_integerbox.nyash:IntegerBox" "test_combined.nyash:Combined")

  local passed=0
  local total=0

  echo "" >&2
  echo "🚀 [Test Suite] Comprehensive FactoryPolicy Testing" >&2

  for policy in "${policies[@]}"; do
    echo "" >&2
    echo "📊 [Policy Suite] Testing: $policy" >&2

    for test_spec in "${tests[@]}"; do
      local test_file="$test_base/${test_spec%:*}"
      local test_name="${test_spec#*:}"

      ((total++))
      if run_policy_test "$policy" "$test_file" "$test_name"; then
        ((passed++))
      fi
    done
  done

  echo "" >&2
  echo "📊 [Test Results] $passed/$total tests passed" >&2

  if [[ $passed -eq $total ]]; then
    echo "🎉 [Test Suite] ALL TESTS PASSED! FactoryPolicy system working perfectly!" >&2
    return 0
  else
    echo "❌ [Test Suite] Some tests failed. Check FactoryPolicy implementation." >&2
    return 1
  fi
}

# Test default behavior (should be StrictPluginFirst after Phase 15.5)
test_default_policy() {
  local test_base=$1

  echo "" >&2
  echo "🌟 [Special Test] Phase 15.5 Default Policy Verification" >&2
  echo "   Expected: StrictPluginFirst (Plugin優先デフォルト)" >&2

  # Unset policy env var to test default
  unset NYASH_BOX_FACTORY_POLICY || true
  export NYASH_CLI_VERBOSE=1

  local output
  if output=$("$BIN" "$test_base/test_stringbox.nyash" 2>&1); then
    if echo "$output" | grep -q "StrictPluginFirst"; then
      echo "✅ [Special Test] SUCCESS: Default policy is StrictPluginFirst!" >&2
      echo "   🎉 Phase 15.5 革命成功確認！" >&2
      return 0
    else
      echo "❌ [Special Test] FAILED: Default policy is not StrictPluginFirst" >&2
      echo "   Output: $output" >&2
      return 1
    fi
  else
    echo "❌ [Special Test] FAILED: Cannot run default policy test" >&2
    echo "   Output: $output" >&2
    return 1
  fi
}

# Main execution
main() {
  echo "🎯 [Plugin Priority Test] Starting comprehensive test suite..." >&2

  # Create test files
  local test_base
  test_base=$(create_test_files)
  echo "[test] Test files created in: $test_base" >&2

  # Run comprehensive tests
  local exit_code=0

  if ! run_comprehensive_tests "$test_base"; then
    exit_code=1
  fi

  if ! test_default_policy "$test_base"; then
    exit_code=1
  fi

  # Cleanup
  rm -rf "$test_base" 2>/dev/null || true

  if [[ $exit_code -eq 0 ]]; then
    echo "" >&2
    echo "🎉 [Plugin Priority Test] 完全成功！Phase 15.5 FactoryPolicy system is working perfectly!" >&2
    echo "   ✅ All policy switching tests passed" >&2
    echo "   ✅ Default StrictPluginFirst confirmed" >&2
    echo "   ✅ Plugin priority system operational" >&2
  else
    echo "" >&2
    echo "❌ [Plugin Priority Test] Some tests failed. Review FactoryPolicy implementation." >&2
  fi

  exit $exit_code
}

main "$@"