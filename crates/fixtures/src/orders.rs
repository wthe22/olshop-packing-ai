//! The made-up orders of the label fixtures. Every value is invented; none comes from `samples/`.

/// How a page's text is cut into text objects. The downloads hold one character per object; the
/// Shopee-style label's edge texts make its pages hold whole strings, which `packing-pdf` reads on
/// another path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Glyphs,
    Words,
}

/// One row of a packing slip. A `*_lines` list fixes where the cell wraps; without one the cell
/// wraps on spaces at its column's width.
pub struct Row {
    pub name: &'static str,
    pub variation: &'static str,
    pub seller_sku: &'static str,
    pub quantity: u32,
    pub name_lines: Option<&'static [&'static str]>,
    pub variation_lines: Option<&'static [&'static str]>,
    pub seller_sku_lines: Option<&'static [&'static str]>,
}

/// The packing slip printed under a label.
pub struct Slip {
    pub rows: Vec<Row>,
    /// The printed `Qty Total`, which need not be the rows' sum.
    pub qty_total: u32,
    /// Printed on a continuation page after the label page.
    pub message: Option<&'static str>,
}

/// One order's label as the fixture prints it, and what reading it must give.
pub struct Label {
    pub order_id: String,
    pub tracking_id: &'static str,
    /// The courier the reader names: from the tracking-ID form and, for J&T, the printed site.
    pub courier: &'static str,
    pub prints_jet_site: bool,
    /// `In transit by`, as printed: `dd/mm/yyyy HH:MM`.
    pub ship_by: Option<String>,
    /// Every character followed by a space, as some labels print their text.
    pub spaced: bool,
    pub slip: Option<Slip>,
    pub style: Style,
}

impl Label {
    pub fn pages(&self) -> usize {
        match &self.slip {
            Some(Slip {
                message: Some(_), ..
            }) => 2,
            _ => 1,
        }
    }
}

struct Product {
    name: &'static str,
    seller_sku: &'static str,
}

const SEPATU: Product = Product {
    name: "Sepatu Standar Samping Motor | Pelindung Rantai",
    seller_sku: "SELL-SEP-01",
};
const SPION: Product = Product {
    name: "Spion Beat | Kaca Spion Motor Honda Beat",
    seller_sku: "SELL-SPI-01",
};
const KNALPOT: Product = Product {
    name: "Knalpot Racing Beat Full System",
    seller_sku: "",
};
const COVER: Product = Product {
    name: "Cover Body Mio | Plastik Body Motor",
    seller_sku: "",
};
const JOK: Product = Product {
    name: "Jok Kulit Vario | Custom",
    seller_sku: "SELL-JOK-01",
};
const FOOTSTEP: Product = Product {
    name: "Footstep Motor | Aluminium",
    seller_sku: "",
};

#[derive(Clone, Copy)]
enum Courier {
    Jnt,
    SiCepat,
    Idx,
}

/// (Order ID suffix, tracking ID, courier, lines as (product, quantity, variation)).
type Spec = (
    &'static str,
    &'static str,
    Courier,
    &'static [(&'static Product, u32, &'static str)],
);

const SLIP_ORDERS: [Spec; 21] = [
    (
        "001",
        "JY0000000101",
        Courier::Jnt,
        &[(&SEPATU, 1, "honda")],
    ),
    (
        "002",
        "000000000202",
        Courier::SiCepat,
        &[(&SEPATU, 1, "HONDA")],
    ),
    (
        "003",
        "JY0000000103",
        Courier::Jnt,
        &[(&SEPATU, 2, "honda")],
    ),
    (
        "004",
        "JY0000000104",
        Courier::Jnt,
        &[(&SPION, 1, "Standard")],
    ),
    (
        "005",
        "000000000205",
        Courier::SiCepat,
        &[(&SPION, 1, "Standard")],
    ),
    (
        "006",
        "TKP0000000106",
        Courier::Idx,
        &[(&SPION, 1, "Standard"), (&KNALPOT, 1, "Default")],
    ),
    // 006's lines in the other order: the same content.
    (
        "007",
        "JY0000000107",
        Courier::Jnt,
        &[(&KNALPOT, 1, "Default"), (&SPION, 1, "HONDA")],
    ),
    (
        "009",
        "000000000209",
        Courier::SiCepat,
        &[(&COVER, 3, "Default")],
    ),
    (
        "010",
        "000000000210",
        Courier::SiCepat,
        &[(&JOK, 3, "Default")],
    ),
    (
        "011",
        "000000000211",
        Courier::SiCepat,
        &[(&COVER, 3, "Default")],
    ),
    (
        "012",
        "JY0000000112",
        Courier::Jnt,
        &[(&COVER, 1, "Default")],
    ),
    ("013", "JY0000000113", Courier::Jnt, &[(&JOK, 1, "Default")]),
    (
        "014",
        "000000000214",
        Courier::SiCepat,
        &[(&FOOTSTEP, 1, "Default")],
    ),
    (
        "015",
        "JY0000000115",
        Courier::Jnt,
        &[(&SEPATU, 1, "honda")],
    ),
    (
        "016",
        "000000000216",
        Courier::SiCepat,
        &[(&SEPATU, 1, "honda")],
    ),
    (
        "017",
        "JY0000000117",
        Courier::Jnt,
        &[(&SEPATU, 2, "honda")],
    ),
    (
        "018",
        "JY0000000118",
        Courier::Jnt,
        &[(&SPION, 1, "Standard")],
    ),
    (
        "019",
        "000000000219",
        Courier::SiCepat,
        &[(&SPION, 1, "Standard")],
    ),
    (
        "020",
        "TKP0000000120",
        Courier::Idx,
        &[(&SPION, 1, "Standard"), (&KNALPOT, 1, "Default")],
    ),
    (
        "021",
        "000000000221",
        Courier::SiCepat,
        &[(&JOK, 3, "Default")],
    ),
    (
        "022",
        "000000000222",
        Courier::SiCepat,
        &[(&FOOTSTEP, 1, "Default")],
    ),
];

fn order_id(suffix: &str) -> String {
    format!("580000000000000{suffix}")
}

fn row(product: &Product, quantity: u32, variation: &'static str) -> Row {
    Row {
        name: product.name,
        variation,
        seller_sku: product.seller_sku,
        quantity,
        name_lines: None,
        variation_lines: None,
        seller_sku_lines: None,
    }
}

/// The orders of `labels-slip.pdf`, in page order.
pub fn slip_labels() -> Vec<Label> {
    let mut labels: Vec<Label> = SLIP_ORDERS
        .iter()
        .map(|&(suffix, tracking_id, courier, lines)| {
            let rows: Vec<Row> = lines
                .iter()
                .map(|&(product, quantity, variation)| row(product, quantity, variation))
                .collect();
            let qty_total = rows.iter().map(|row| row.quantity).sum();
            let number: u32 = suffix.parse().expect("numeric suffix");
            Label {
                order_id: order_id(suffix),
                tracking_id,
                courier: match courier {
                    Courier::Jnt => "J&T Express",
                    Courier::SiCepat => "",
                    Courier::Idx => "IDX",
                },
                prints_jet_site: matches!(courier, Courier::Jnt),
                ship_by: Some(format!("07/10/2026 {:02}:15", 10 + number % 6)),
                spaced: false,
                slip: Some(Slip {
                    rows,
                    qty_total,
                    message: (suffix == "004").then_some("Tolong bubble wrap, jangan dilipat"),
                }),
                style: Style::Glyphs,
            }
        })
        .collect();
    // Cells that wrap right after '-', and a printed total that is not the rows' sum.
    labels.push(Label {
        order_id: order_id("901"),
        tracking_id: "JY0000000901",
        courier: "J&T Express",
        prints_jet_site: true,
        ship_by: Some("07/10/2026 12:45".into()),
        spaced: false,
        slip: Some(Slip {
            rows: vec![
                Row {
                    name: "Cover Knalpot Beat FI 2012-2015 Full Set",
                    variation: "Yamaha-Mio",
                    seller_sku: "SPION-SCOL-DH-CY",
                    quantity: 1,
                    name_lines: Some(&["Cover Knalpot Beat", "FI 2012-", "2015 Full Set"]),
                    variation_lines: Some(&["Yamaha-", "Mio"]),
                    seller_sku_lines: Some(&["SPION-SCOL-DH-", "CY"]),
                },
                row(
                    &Product {
                        name: "Spion Beat Samping",
                        seller_sku: "",
                    },
                    1,
                    "Default",
                ),
            ],
            qty_total: 3,
            message: None,
        }),
        style: Style::Glyphs,
    });
    // A 12-digit tracking ID and no courier text: the courier stays unknown.
    labels.push(Label {
        order_id: order_id("902"),
        tracking_id: "000000000902",
        courier: "",
        prints_jet_site: false,
        ship_by: None,
        spaced: false,
        slip: Some(Slip {
            rows: vec![row(
                &Product {
                    name: "Sepatu Standar",
                    seller_sku: "SELL-SEP-01",
                },
                2,
                "honda",
            )],
            qty_total: 2,
            message: None,
        }),
        style: Style::Glyphs,
    });
    labels
}

/// The orders of `labels-plain.pdf`: labels without a packing slip. The last page is drawn in
/// whole strings, so the reader's second path has a fixture too.
pub fn plain_labels() -> Vec<Label> {
    let plain = |suffix: &str, tracking_id, courier, ship_by: Option<&str>| Label {
        order_id: order_id(suffix),
        tracking_id,
        courier,
        prints_jet_site: courier == "J&T Express",
        ship_by: ship_by.map(str::to_string),
        spaced: false,
        slip: None,
        style: Style::Glyphs,
    };
    vec![
        plain(
            "701",
            "JY0000000701",
            "J&T Express",
            Some("07/10/2026 09:00"),
        ),
        Label {
            spaced: true,
            ..plain("702", "TKP0000000702", "IDX", Some("07/10/2026 09:30"))
        },
        Label {
            style: Style::Words,
            ..plain("703", "000000000703", "", None)
        },
    ]
}

/// The two inputs of `labels-repeat-1.pdf` and `labels-repeat-2.pdf`: order 102 printed twice in a
/// row, each copy a label page and a continuation page; order 103 at the end of the first input
/// and again at the start of the second.
pub fn repeat_inputs() -> [Vec<Label>; 2] {
    let label = |suffix: &str, tracking_id, courier, product: &Product, quantity, message| Label {
        order_id: order_id(suffix),
        tracking_id,
        courier,
        prints_jet_site: courier == "J&T Express",
        ship_by: Some("08/10/2026 23:59".into()),
        spaced: false,
        slip: Some(Slip {
            rows: vec![row(product, quantity, "Default")],
            qty_total: quantity,
            message,
        }),
        style: Style::Glyphs,
    };
    let second = || {
        label(
            "102",
            "JY0000000202",
            "J&T Express",
            &SPION,
            1,
            Some("Jangan dibanting"),
        )
    };
    let last = || label("103", "000000000203", "", &JOK, 1, None);
    [
        vec![
            label("101", "JY0000000201", "J&T Express", &SEPATU, 1, None),
            second(),
            second(),
            last(),
        ],
        vec![
            last(),
            label("104", "TKP0000000204", "IDX", &KNALPOT, 2, None),
        ],
    ]
}
