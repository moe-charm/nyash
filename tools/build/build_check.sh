#!/bin/bash
cargo check --lib > build_errors.txt 2>&1
echo "Build check completed, errors saved to build_errors.txt"