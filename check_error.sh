#!/bin/bash
cargo check 2>&1 | grep -B2 -A2 "unresolved import"