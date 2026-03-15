use futures::executor::block_on;

use acpx::{
    AgentServer, Error, RuntimeContext, UnsupportedLaunch,
    agent_servers::REGISTRY_VERSION,
    registry::{
        HostPlatform, RegistryError, agent_server, agent_servers, binary_target_for,
        registry_version, require_agent_server,
    },
};

#[test]
fn agent_server_resolves_official_registry_ids() {
    let codex = agent_server("codex-acp").expect("codex-acp should resolve");
    assert_eq!(codex.id(), "codex-acp");

    let droid = agent_server("factory-droid").expect("factory-droid should resolve");
    assert_eq!(droid.id(), "factory-droid");

    assert!(agent_server("missing-agent").is_none());
    assert!(matches!(
        require_agent_server("missing-agent"),
        Err(RegistryError::UnknownAgentServer { id }) if id == "missing-agent"
    ));
}

#[test]
fn agent_servers_match_the_generated_snapshot() {
    let registry_agent_servers = agent_servers();
    assert!(!registry_agent_servers.is_empty());
    assert_eq!(registry_version(), REGISTRY_VERSION);
    assert!(
        registry_agent_servers
            .iter()
            .any(|agent_server| agent_server.id() == "codex-acp")
    );
    assert!(
        registry_agent_servers
            .iter()
            .any(|agent_server| agent_server.id() == "factory-droid")
    );
    assert!(
        registry_agent_servers
            .iter()
            .any(|agent_server| agent_server.id() == "autohand")
    );
}

#[test]
fn host_platform_mapping_covers_known_registry_targets() {
    assert_eq!(
        HostPlatform::from_target("macos", "aarch64").expect("macOS arm64 should map"),
        HostPlatform::DarwinAarch64
    );
    assert_eq!(
        HostPlatform::from_target("macos", "x86_64").expect("macOS x86_64 should map"),
        HostPlatform::DarwinX86_64
    );
    assert_eq!(
        HostPlatform::from_target("linux", "aarch64").expect("linux arm64 should map"),
        HostPlatform::LinuxAarch64
    );
    assert_eq!(
        HostPlatform::from_target("linux", "x86_64").expect("linux x86_64 should map"),
        HostPlatform::LinuxX86_64
    );
    assert_eq!(
        HostPlatform::from_target("windows", "aarch64").expect("windows arm64 should map"),
        HostPlatform::WindowsAarch64
    );
    assert_eq!(
        HostPlatform::from_target("windows", "x86_64").expect("windows x86_64 should map"),
        HostPlatform::WindowsX86_64
    );
    assert!(matches!(
        HostPlatform::from_target("freebsd", "x86_64"),
        Err(RegistryError::UnsupportedHostPlatform { os, arch })
            if os == "freebsd" && arch == "x86_64"
    ));
}

#[test]
fn binary_target_resolution_matches_platform_support() {
    let corust_agent_server = agent_server("corust-agent").expect("corust-agent should resolve");
    let linux_x86_64_target = binary_target_for(&corust_agent_server, HostPlatform::LinuxX86_64)
        .expect("linux x86_64 should exist")
        .expect("binary target should resolve");
    assert_eq!(linux_x86_64_target.target(), "linux-x86_64");

    assert!(matches!(
        binary_target_for(&corust_agent_server, HostPlatform::LinuxAarch64),
        Err(RegistryError::MissingBinaryTarget { id, target })
            if id == "corust-agent" && target == "linux-aarch64"
    ));

    let autohand_agent_server = agent_server("autohand").expect("autohand should resolve");
    assert!(
        binary_target_for(&autohand_agent_server, HostPlatform::LinuxX86_64)
            .expect("package-backed agent servers should not fail")
            .is_none()
    );
}

#[test]
fn binary_only_generated_servers_return_typed_unsupported_launch_errors() {
    let runtime = RuntimeContext::new(|_| {});
    let amp_agent_server = require_agent_server("amp-acp").expect("amp-acp should resolve");
    let error = block_on(amp_agent_server.connect(&runtime)).err();

    assert!(matches!(
        error,
        Some(Error::UnsupportedLaunch(
            UnsupportedLaunch::BinaryDistribution
        ))
    ));
}
