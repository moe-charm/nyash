#[cfg(feature = "aot-plan-import")]
pub mod aot_plan_import;
pub mod box_tests;
pub mod host_reverse_slot;
pub mod identical_exec;
pub mod identical_exec_collections;
pub mod identical_exec_instance;
pub mod identical_exec_string;
#[cfg(feature = "mir_typeop_poc")]
pub mod mir_vm_poc;
pub mod nyash_abi_basic;
#[cfg(feature = "cranelift-jit")] pub mod plugin_hygiene;
#[cfg(feature = "cranelift-jit")] pub mod policy_mutdeny;
pub mod sugar_basic_test;
pub mod sugar_coalesce_test;
pub mod sugar_comp_assign_test;
pub mod sugar_pipeline_test;
pub mod sugar_range_test;
pub mod sugar_safe_access_test;
pub mod typebox_tlv_diff;
#[cfg(feature = "builtin-core")] pub mod vtable_array_ext;
#[cfg(feature = "builtin-core")] pub mod vtable_array_p1;
#[cfg(feature = "builtin-core")] pub mod vtable_array_p2;
#[cfg(feature = "builtin-core")] pub mod vtable_array_string;
#[cfg(feature = "builtin-core")] pub mod vtable_console;
#[cfg(feature = "builtin-core")] pub mod vtable_map_ext;
pub mod vtable_strict;
#[cfg(feature = "builtin-core")] pub mod vtable_string;
#[cfg(feature = "builtin-core")] pub mod vtable_string_p1;
