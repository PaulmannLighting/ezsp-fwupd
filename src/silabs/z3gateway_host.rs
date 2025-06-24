use std::process::{Command, Stdio};

const Z3GATEWAY_HOST: &str = "/usr/bin/Z3GatewayHost";

/// Represents a host for the Z3 Gateway, which is used to communicate with Silicon Labs devices.
pub trait Z3GatewayHost {
    fn z3gateway_host() -> Self;
}

impl Z3GatewayHost for Command {
    /// Creates a new instance of `Z3GatewayHost`.
    fn z3gateway_host() -> Self {
        let mut command = Self::new(Z3GATEWAY_HOST);
        command
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());
        command
    }
}
