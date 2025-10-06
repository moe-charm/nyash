/*!\
 Using system — resolution scaffolding (Phase 15 skeleton)\
\
 Centralizes name/path resolution for `using` statements.\
 This initial cut only reads nyash.toml to populate:\
  - [using.paths]  → search roots for source lookups\
  - [modules]      → logical name → file path mapping\
  - [aliases]      → convenience alias mapping (optional)\
\
 The goal is to keep runner/pipeline lean by delegating nyash.toml parsing here,\
 without changing default behavior. Future work will add: file/DLL specs, policies,\
 and plugin metadata fusion (hako_box.toml or nyash_box.toml / embedded BID).\
*/

pub mod resolver;
pub mod spec;
pub mod policy;
pub mod errors;
pub mod simple_registry;
pub mod namespace_box;
