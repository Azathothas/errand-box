//! The backend-agnostic egress gate.
//!
//! The policy a session's egress is judged by, apart from how its traffic
//! reaches the broker. Today the HTTP proxy in [`super::server`] feeds it;
//! T-090's virtual network layer will feed the same decisions from a userspace
//! netstack, so they live here rather than inline in the transport, and a
//! backend calls the gate instead of reaching into the proxy. The host and
//! address predicates it builds on (`host_allowed`, `is_private_address`,
//! `public_addresses`) are already free of the transport and stay beside the
//! request parser in the parent module.

use super::{ProviderRoute, host_allowed};

/// Whether a target's host and port are admitted, and if not, why.
///
/// The host allowlist and the port list are separate causes, named separately
/// in the refusal so a refused session learns which to fix (T-083). This is the
/// verdict the proxy used to build inline in `admit`; any backend reuses it
/// unchanged.
pub(crate) fn target_admission(
    allow: &[String],
    ports: &[u16],
    host: &str,
    port: u16,
) -> Result<(), String> {
    let host_ok = host_allowed(host, allow);
    let port_ok = ports.contains(&port);
    if host_ok && port_ok {
        return Ok(());
    }
    Err(if !host_ok && !port_ok {
        format!("{host} is not on the egress allowlist, and port {port} is not open to egress")
    } else if !host_ok {
        format!("{host} is not on the egress allowlist")
    } else {
        format!("port {port} is not open to egress")
    })
}

/// The provider route that serves `path`, longest matching prefix first.
pub(crate) fn route_for<'a>(routes: &'a [ProviderRoute], path: &str) -> Option<&'a ProviderRoute> {
    let mut routes = routes.iter().collect::<Vec<_>>();
    // Longest prefix first, so a provider named under another's path is still
    // reached rather than shadowed by it.
    routes.sort_by_key(|route| std::cmp::Reverse(route.prefix.len()));
    routes
        .into_iter()
        .find(|candidate| under_prefix(path, &candidate.prefix))
}

/// Whether `path` is `prefix` itself or a path below it.
///
/// A match on the bare string would let `/provider/NAME.other.example/...` and
/// `/provider/NAME:443@other.example/...` reach the route for `NAME`, and what
/// follows the prefix is pasted onto the provider's address.
pub(crate) fn under_prefix(path: &str, prefix: &str) -> bool {
    let prefix = prefix.strip_suffix('/').unwrap_or(prefix);
    path.strip_prefix(prefix)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}
