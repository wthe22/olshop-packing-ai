use super::{Layout, Scope};

/// The packing-list names the command line and the packing-list file names use: parsing
/// accepts exactly the names the two enums write.
#[test]
fn packing_list_names_round_trip() {
    for (name, scope) in [
        ("whole", Scope::Whole),
        ("per-pdf", Scope::PerPdf),
        ("per_pdf", Scope::PerPdf),
        ("perpdf", Scope::PerPdf),
        ("none", Scope::None),
    ] {
        assert_eq!(name.parse::<Scope>(), Ok(scope), "{name}");
    }
    assert!("nope".parse::<Scope>().is_err());

    for (name, layout) in [
        ("pick", Layout::Pick),
        ("full", Layout::Full),
        ("summary", Layout::Summary),
    ] {
        assert_eq!(name.parse::<Layout>(), Ok(layout), "{name}");
        assert_eq!(layout.as_str(), name);
    }
    assert!("nope".parse::<Layout>().is_err());
}
