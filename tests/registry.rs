use futures::executor::block_on;

use acpx::{
    AgentServer, Error, RuntimeContext, UnsupportedLaunch,
    agent_servers::REGISTRY_VERSION,
    registry::{
        HostPlatform, RegistryError, alias_target, binary_target_for, curated_agent_servers,
        lookup, registry_version, require,
    },
};

#[test]
fn curated_lookup_resolves_ids_and_aliases() {
    let copilot = lookup("copilot").expect("copilot alias should resolve");
    assert_eq!(copilot.id(), "github-copilot-cli");
    assert_eq!(alias_target("copilot"), Some("github-copilot-cli"));

    let direct = lookup("github-copilot-cli").expect("canonical id should resolve");
    assert_eq!(direct.id(), "github-copilot-cli");
    assert!(lookup("github-copilot").is_none());
    assert!(matches!(
        require("missing-agent"),
        Err(RegistryError::UnknownAgent { id }) if id == "missing-agent"
    ));
}

#[test]
fn curated_listing_stays_on_the_generated_snapshot() {
    let curated = curated_agent_servers();
    assert!(!curated.is_empty());
    assert_eq!(registry_version(), REGISTRY_VERSION);
    assert!(
        curated
            .iter()
            .any(|agent| agent.id() == "github-copilot-cli")
    );
    assert!(curated.iter().all(|agent| agent.id() != "github-copilot"));
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
    let corust = lookup("corust-agent").expect("corust-agent should resolve");
    let target = binary_target_for(&corust, HostPlatform::LinuxX86_64)
        .expect("linux x86_64 should exist")
        .expect("binary target should resolve");
    assert_eq!(target.target(), "linux-x86_64");

    assert!(matches!(
        binary_target_for(&corust, HostPlatform::LinuxAarch64),
        Err(RegistryError::MissingBinaryTarget { id, target })
            if id == "corust-agent" && target == "linux-aarch64"
    ));

    let autohand = lookup("autohand").expect("autohand should resolve");
    assert!(
        binary_target_for(&autohand, HostPlatform::LinuxX86_64)
            .expect("package-backed agents should not fail")
            .is_none()
    );
}

#[test]
fn binary_only_generated_servers_return_typed_unsupported_launch_errors() {
    let runtime = RuntimeContext::new(|_| {});
    let amp = require("amp-acp").expect("amp-acp should resolve");
    let error = block_on(amp.connect(&runtime)).err();

    assert!(matches!(
        error,
        Some(Error::UnsupportedLaunch(UnsupportedLaunch::BinaryDistribution))
    ));
}
