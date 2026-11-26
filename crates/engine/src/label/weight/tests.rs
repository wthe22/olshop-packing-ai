use super::find_weight;
use crate::record::Weight;
use crate::text::{PageText, Run};

fn page(runs: &[(f64, f64, &str)]) -> PageText {
    PageText {
        runs: runs
            .iter()
            .map(|&(x, y, text)| Run {
                x,
                y,
                text: text.to_string(),
            })
            .collect(),
        ..PageText::default()
    }
}

#[test]
fn each_layout_gives_the_weight_it_prints_in_grams() {
    let cases = [
        (
            "TikTok: the caption alone, the value right of it",
            page(&[(19.7, 282.3, "Weight :"), (53.2, 282.5, "0.490 KG")]),
            Weight::Grams(490),
        ),
        (
            "TikTok: the caption in the date's run",
            page(&[
                (169.9, 282.6, "06-10-2026Weight :"),
                (53.2, 282.6, "1.000 KG"),
            ]),
            Weight::Grams(1000),
        ),
        (
            "TikTok: fewer decimals",
            page(&[(19.7, 282.3, "Weight :"), (53.2, 282.3, "0.5 KG")]),
            Weight::Grams(500),
        ),
        (
            "TikTok boxed: no colon, whole kilograms",
            page(&[(11.4, 165.1, "Weight"), (41.2, 165.5, "5 KG")]),
            Weight::Grams(5000),
        ),
        (
            "TikTok: caption and value in one run",
            page(&[(19.7, 282.3, "Weight : 0.300 KG")]),
            Weight::Grams(300),
        ),
        (
            "a product name with the caption's word above the caption",
            page(&[
                (5.8, 300.0, "Weight Lifting Belt"),
                (19.7, 282.3, "Weight :"),
                (53.2, 282.3, "0.150 KG"),
            ]),
            Weight::Grams(150),
        ),
        (
            "Shopee-style: the value 1.5 pt lower",
            page(&[(12.7, 223.8, "Berat:"), (39.6, 222.3, "450 gr")]),
            Weight::Grams(450),
        ),
        (
            "Shopee-style: caption and value in one run",
            page(&[(12.7, 235.0, "Berat: 1000 gr")]),
            Weight::Grams(1000),
        ),
        (
            "instant: no caption",
            page(&[(19.0, 344.1, "Alamat :"), (53.2, 282.6, "0.500 KG")]),
            Weight::NotPrinted,
        ),
        (
            "a value on another line",
            page(&[(19.7, 282.3, "Weight :"), (53.2, 270.0, "0.500 KG")]),
            Weight::NotPrinted,
        ),
        (
            "a value that is not a number",
            page(&[(19.7, 282.3, "Weight :"), (53.2, 282.3, "0.5.0 KG")]),
            Weight::NotPrinted,
        ),
        (
            "more than three decimals",
            page(&[(19.7, 282.3, "Weight :"), (53.2, 282.3, "0.4905 KG")]),
            Weight::NotPrinted,
        ),
    ];
    for (case, page, wanted) in cases {
        assert_eq!(find_weight(&page), wanted, "{case}");
    }
}
