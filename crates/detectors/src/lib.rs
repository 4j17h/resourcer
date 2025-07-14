//! Runtime detector plugins live here. For now this crate is empty and compiles as part of the workspace.

// A trivial function to satisfy `cargo test`.
#[allow(dead_code)]
pub fn detectors_placeholder() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_compiles() {
        detectors_placeholder();
    }
}
