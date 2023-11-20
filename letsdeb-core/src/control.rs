use debcontrol::{Paragraph, Field};
use debcontrol_struct::DebControl;

#[derive(Debug, DebControl)]
struct Control {
    package: String,
    version: String,
    architecture: String,
    section: String,
    maintainer: String,
    installed_size: String,
    description: String,
    essential: String,
    depends: Option<String>,
}

impl Default for Control {
    fn default() -> Self {
        Self {
            package: "test".to_string(),
            version: "0.0".to_string(),
            architecture: "all".to_string(),
            section: "utils".to_string(),
            maintainer: "tester <tester@aosc.io>".to_string(),
            installed_size: 0.to_string(),
            description: "Test package".to_string(),
            essential: "no".to_string(),
            depends: None,
        }
    }
}
