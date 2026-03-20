use std::env;
use std::sync::Once;

// Initialization flag to ensure we only initialize once
static INIT: Once = Once::new();

// Environment variable to control enhanced output
const ENV_ENHANCED_OUTPUT: &str = "REST_ENHANCED_OUTPUT";
const DEFAULT_ENHANCED_OUTPUT: bool = true;

/// Configuration for Rest's output and behavior
pub struct Config {
    pub(crate) use_colors: bool,
    pub(crate) use_unicode_symbols: bool,
    pub(crate) show_success_details: bool,
    /// Enable enhanced test output (fluent assertions instead of standard output)
    pub(crate) enhanced_output: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Clone for Config
impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            use_colors: self.use_colors,
            use_unicode_symbols: self.use_unicode_symbols,
            show_success_details: self.show_success_details,
            enhanced_output: self.enhanced_output,
        }
    }
}

impl Config {
    /// Creates a new configuration with default settings
    pub fn new() -> Self {
        // Check for environment variable to enable enhanced output.
        let enhanced_output = match env::var(ENV_ENHANCED_OUTPUT).ok() {
            Some(val) => bool_from_str(&val, DEFAULT_ENHANCED_OUTPUT),
            None => DEFAULT_ENHANCED_OUTPUT,
        };

        Self { use_colors: true, use_unicode_symbols: true, show_success_details: true, enhanced_output }
    }

    /// Enable or disable colored output
    pub fn use_colors(mut self, enable: bool) -> Self {
        self.use_colors = enable;
        self
    }

    /// Enable or disable Unicode symbols
    pub fn use_unicode_symbols(mut self, enable: bool) -> Self {
        self.use_unicode_symbols = enable;
        self
    }

    /// Control whether to show details for successful tests
    pub fn show_success_details(mut self, enable: bool) -> Self {
        self.show_success_details = enable;
        self
    }

    /// Enable or disable enhanced output (fluent assertions)
    pub fn enhanced_output(mut self, enable: bool) -> Self {
        self.enhanced_output = enable;
        self
    }

    /// Apply the configuration
    pub fn apply(self) {
        use crate::reporter::GLOBAL_CONFIG;

        // Clone self before moving it into the global config
        let config = self.clone();
        *GLOBAL_CONFIG.write().unwrap() = self;

        // Initialize the event system if enhanced output is enabled
        if config.enhanced_output {
            crate::initialize();
        }
    }
}

/// Initialize the Rest system
/// This is called automatically when needed but can also be called explicitly
pub fn initialize() {
    INIT.call_once(|| {
        // Check if enhanced output is enabled in the config
        let config = crate::reporter::GLOBAL_CONFIG.read().unwrap();

        if config.enhanced_output {
            // Initialize event system
            crate::events::EventEmitter::init();

            // Register event handlers
            crate::Reporter::init();
        }
    });
}

/// Check if enhanced output is enabled in the current configuration
pub fn is_enhanced_output_enabled() -> bool {
    let config = crate::reporter::GLOBAL_CONFIG.read().unwrap();
    return config.enhanced_output;
}

/// Convert from one of the allowed string values to a boolean.
fn bool_from_str(val: &str, default: bool) -> bool {
    match val.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => {
            use colored::Colorize;
            let warning = "WARNING".yellow();
            let val_str = val.red();
            let default_str = format!("{}", default).blue();
            let allowed: Vec<_> =
                ["true", "false", "1", "0", "yes", "no", "on", "off"].into_iter().map(|it| it.green().to_string()).collect();
            let allowed = allowed.join(", ");
            eprintln!(
                "{warning}: Unrecognized value for environment variable \
                {ENV_ENHANCED_OUTPUT}: {val_str}.\n\
                Defaulting to {default_str}.\n\
                (Allowed values: {allowed})",
            );
            default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Helper functions set/reset environment variables
    // XXX: These are _truly_ unsafe in multithreaded programs on UNIX, which is probably why tests randomly crash.
    fn set_env_var(value: &str) {
        unsafe {
            env::set_var(ENV_ENHANCED_OUTPUT, value);
        }
    }

    fn reset_env_var() {
        unsafe {
            env::remove_var(ENV_ENHANCED_OUTPUT);
        }
    }

    #[test]
    fn test_config_default() {
        // Make sure we start with a clean environment
        reset_env_var();

        let config = Config::new();

        // Check default values
        assert_eq!(config.use_colors, true);
        assert_eq!(config.use_unicode_symbols, true);
        assert_eq!(config.show_success_details, true);
        assert_eq!(config.enhanced_output, true, "Default is true without env var");
    }

    #[test]
    fn test_config_env_var_true() {
        // Test with environment variable set to true
        reset_env_var();
        set_env_var("true");

        let config = Config::new();
        assert_eq!(config.enhanced_output, true);

        // Cleanup
        reset_env_var();
    }

    #[test]
    fn test_config_env_var_false() {
        // Test with environment variable set to false
        set_env_var("false");

        let config = Config::new();
        assert_eq!(config.enhanced_output, false);

        // Cleanup
        reset_env_var();
    }

    #[test]
    #[ignore] // This test is ignored because sometimes it fails on Github Actions
    fn test_config_env_var_alternative_values() {
        // True values
        set_env_var("1");
        assert_eq!(Config::new().enhanced_output, true);

        set_env_var("yes");
        assert_eq!(Config::new().enhanced_output, true);

        set_env_var("on");
        assert_eq!(Config::new().enhanced_output, true);

        set_env_var("0");
        assert_eq!(Config::new().enhanced_output, false);

        set_env_var("no");
        assert_eq!(Config::new().enhanced_output, false);

        set_env_var("off");
        assert_eq!(Config::new().enhanced_output, false);

        // Garbage inputs: Env ignored, use default value.
        set_env_var("uh ... dunno");
        assert_eq!(Config::new().enhanced_output, DEFAULT_ENHANCED_OUTPUT);

        // Test case-insensitivity
        set_env_var("TRUE");
        assert_eq!(Config::new().enhanced_output, true);

        // Cleanup
        reset_env_var();
    }

    #[test]
    fn test_config_builder_methods() {
        let config = Config::new().use_colors(false).use_unicode_symbols(false).show_success_details(false).enhanced_output(true);

        assert_eq!(config.use_colors, false);
        assert_eq!(config.use_unicode_symbols, false);
        assert_eq!(config.show_success_details, false);
        assert_eq!(config.enhanced_output, true);
    }

    #[test]
    fn test_config_clone() {
        let config1 = Config::new().use_colors(false).enhanced_output(true);

        let config2 = config1.clone();

        // Make sure the clone has the same values
        assert_eq!(config1.use_colors, config2.use_colors);
        assert_eq!(config1.use_unicode_symbols, config2.use_unicode_symbols);
        assert_eq!(config1.show_success_details, config2.show_success_details);
        assert_eq!(config1.enhanced_output, config2.enhanced_output);
    }

    // Note: Testing apply() and initialize() would require mocking or complex setups
    // since they interact with global state. For a unit test, we're focusing on the
    // pure functionality that can be tested in isolation.
}
