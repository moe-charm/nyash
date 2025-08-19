#!/bin/bash
./target/release/nyash local_tests/test_filebox_debug.nyash 2>&1 | grep -E "(TLV data|Plugin method returned)"