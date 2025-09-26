# E2E Tests Overview

Purpose
- Track end-to-end coverage with plugins and core features, both interpreter and VM.

HTTP (plugins)
- GET basic (VM): `e2e_vm_http_get_basic` → body `OK`
- POST + headers (VM): `e2e_vm_http_post_and_headers` → `201:V:R`
- Status 404 (VM): `e2e_vm_http_status_404` → `404:NF`
- Status 500 (VM): `e2e_vm_http_status_500` → `500:ERR`
- Client error (unreachable) (VM): `e2e_vm_http_client_error_result` → `Result.Err(ErrorBox)`

FileBox (plugins)
- Close returns void (Interp/VM)
- Open/Write/Read (VM): `e2e_vm_plugin_filebox_open_rw` → `HELLO`
- copyFrom(handle) (VM): `e2e_vm_plugin_filebox_copy_from_handle` → `HELLO`

MIR/VM Core
- Ref ops MIR build: `mir_phase6_lowering_ref_ops`
- Ref ops VM exec: `mir_phase6_vm_ref_ops`
- Async ops MIR/VM: `mir_phase7_async_ops`

Conventions
- Use distinct ports per test (8080+). Enable logs only on failure to keep CI output tidy.
- Plugins logs: `NYASH_NET_LOG=1 NYASH_NET_LOG_FILE=net_plugin.log`.

