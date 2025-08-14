/*!
 * AOT Configuration - Wasmtime optimization settings
 * 
 * Manages compilation settings, CPU features, and performance tuning
 */

use super::AotError;
use wasmtime::{Config, OptLevel, Strategy};

/// AOT compilation configuration
#[derive(Debug, Clone)]
pub struct AotConfig {
    wasmtime_config: Config,
    optimization_level: u8,
    enable_simd: bool,
    enable_bulk_memory: bool,
    enable_multi_memory: bool,
    target_arch: String,
}

impl AotConfig {
    /// Create default configuration optimized for performance
    pub fn new() -> Result<Self, AotError> {
        let mut config = Config::new();
        
        // Enable maximum optimizations
        config.strategy(Strategy::Cranelift);
        config.cranelift_opt_level(OptLevel::Speed);
        
        // Enable WebAssembly features for better performance
        config.wasm_simd(true);
        config.wasm_bulk_memory(true);
        config.wasm_multi_memory(true);
        
        // Enable advanced optimizations
        unsafe {
            config.cranelift_flag_enable("enable_verifier");
            config.cranelift_flag_enable("enable_nan_canonicalization");
        }
        
        // Set memory limits for safety (64MB max)
        config.max_wasm_stack(8 * 1024 * 1024); // 8MB stack
        
        let target_arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else {
            "unknown"
        }.to_string();
        
        Ok(Self {
            wasmtime_config: config,
            optimization_level: 3, // Maximum optimization
            enable_simd: true,
            enable_bulk_memory: true,
            enable_multi_memory: true,
            target_arch,
        })
    }
    
    /// Create configuration optimized for debug builds
    pub fn debug() -> Result<Self, AotError> {
        let mut config = Config::new();
        
        config.strategy(Strategy::Cranelift);
        config.cranelift_opt_level(OptLevel::None);
        
        // Enable debug features
        config.debug_info(true);
        
        // Basic WASM features only
        config.wasm_simd(false);
        config.wasm_bulk_memory(true);
        
        let target_arch = std::env::consts::ARCH.to_string();
        
        Ok(Self {
            wasmtime_config: config,
            optimization_level: 0,
            enable_simd: false,
            enable_bulk_memory: true,
            enable_multi_memory: false,
            target_arch,
        })
    }
    
    /// Create configuration for specific target architecture
    pub fn for_target(target: &str) -> Result<Self, AotError> {
        let mut config = Self::new()?;
        config.target_arch = target.to_string();
        
        // Adjust features based on target
        match target {
            "x86_64" => {
                // Enable all advanced features for x86_64
                config.enable_simd = true;
                config.enable_multi_memory = true;
            },
            "aarch64" => {
                // ARM64 - enable SIMD but be conservative with memory features
                config.enable_simd = true;
                config.enable_multi_memory = false;
            },
            "x86" => {
                // x86 - be conservative
                config.enable_simd = false;
                config.enable_multi_memory = false;
            },
            _ => {
                return Err(AotError::ConfigError(format!("Unsupported target architecture: {}", target)));
            }
        }
        
        // Rebuild wasmtime config with new settings
        config.rebuild_wasmtime_config()?;
        
        Ok(config)
    }
    
    /// Get the wasmtime configuration
    pub fn wasmtime_config(&self) -> &Config {
        &self.wasmtime_config
    }
    
    /// Get optimization level (0-3)
    pub fn optimization_level(&self) -> u8 {
        self.optimization_level
    }
    
    /// Get target architecture
    pub fn target_arch(&self) -> &str {
        &self.target_arch
    }
    
    /// Check if SIMD is enabled
    pub fn simd_enabled(&self) -> bool {
        self.enable_simd
    }
    
    /// Get compatibility key for cache validation
    pub fn compatibility_key(&self) -> String {
        format!(
            "nyash-aot-{}-opt{}-simd{}-bulk{}-multi{}-wasmtime{}",
            self.target_arch,
            self.optimization_level,
            self.enable_simd,
            self.enable_bulk_memory,
            self.enable_multi_memory,
            "18.0" // Wasmtime version from Cargo.toml
        )
    }
    
    /// Rebuild wasmtime config with current settings
    fn rebuild_wasmtime_config(&mut self) -> Result<(), AotError> {
        let mut config = Config::new();
        
        config.strategy(Strategy::Cranelift);
        
        let opt_level = match self.optimization_level {
            0 => OptLevel::None,
            1 => OptLevel::Speed,
            2 => OptLevel::Speed,
            3 => OptLevel::SpeedAndSize,
            _ => OptLevel::Speed,
        };
        
        config.cranelift_opt_level(opt_level);
        config.wasm_simd(self.enable_simd);
        config.wasm_bulk_memory(self.enable_bulk_memory);
        config.wasm_multi_memory(self.enable_multi_memory);
        
        // Set memory limits
        config.max_wasm_stack(8 * 1024 * 1024); // 8MB stack
        
        if self.optimization_level >= 2 {
            unsafe {
                config.cranelift_flag_enable("enable_verifier");
            }
        }
        
        self.wasmtime_config = config;
        Ok(())
    }
    
    /// Set custom optimization level
    pub fn set_optimization_level(&mut self, level: u8) -> Result<(), AotError> {
        if level > 3 {
            return Err(AotError::ConfigError("Optimization level must be 0-3".to_string()));
        }
        
        self.optimization_level = level;
        self.rebuild_wasmtime_config()
    }
    
    /// Enable or disable SIMD
    pub fn set_simd(&mut self, enabled: bool) -> Result<(), AotError> {
        self.enable_simd = enabled;
        self.rebuild_wasmtime_config()
    }
}

impl Default for AotConfig {
    fn default() -> Self {
        Self::new().expect("Failed to create default AOT config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = AotConfig::new().expect("Failed to create config");
        assert_eq!(config.optimization_level(), 3);
        assert!(config.simd_enabled());
    }
    
    #[test]
    fn test_debug_config() {
        let config = AotConfig::debug().expect("Failed to create debug config");
        assert_eq!(config.optimization_level(), 0);
        assert!(!config.simd_enabled());
    }
    
    #[test]
    fn test_compatibility_key() {
        let config = AotConfig::new().expect("Failed to create config");
        let key = config.compatibility_key();
        assert!(key.contains("nyash-aot"));
        assert!(key.contains("wasmtime"));
    }
    
    #[test]
    fn test_target_config() {
        let config = AotConfig::for_target("x86_64").expect("Failed to create x86_64 config");
        assert_eq!(config.target_arch(), "x86_64");
        assert!(config.simd_enabled());
    }
    
    #[test]
    fn test_optimization_level_setting() {
        let mut config = AotConfig::new().expect("Failed to create config");
        config.set_optimization_level(1).expect("Failed to set opt level");
        assert_eq!(config.optimization_level(), 1);
    }
    
    #[test]
    fn test_invalid_optimization_level() {
        let mut config = AotConfig::new().expect("Failed to create config");
        assert!(config.set_optimization_level(4).is_err());
    }
}