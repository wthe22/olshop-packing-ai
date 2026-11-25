use super::find_area;
use crate::record::Area;
use crate::text::{PageText, Run};

fn run(x: f64, y: f64, text: &str) -> Run {
    Run {
        x,
        y,
        text: text.to_string(),
    }
}

fn page(runs: Vec<Run>) -> PageText {
    PageText {
        runs,
        ..PageText::default()
    }
}

fn area(province: Option<&str>, city: Option<&str>, district: Option<&str>) -> Area {
    Area {
        province: province.map(str::to_string),
        city: city.map(str::to_string),
        district: district.map(str::to_string),
    }
}

#[test]
fn each_layout_gives_its_province_city_and_district_as_printed() {
    let cases = [
        (
            "standard: the line under the caption, the tracking ID's digit beside it",
            page(vec![
                run(19.0, 344.1, "Penerima :"),
                run(13.5, 340.1, "J"),
                run(19.3, 335.1, "JAWA BARAT,KAB. CONTOH,CONTOH TENGAH"),
                run(19.0, 322.6, "Jl. Contoh No. 1, RT 01, RW 02"),
            ]),
            area(
                Some("JAWA BARAT"),
                Some("KAB. CONTOH"),
                Some("CONTOH TENGAH"),
            ),
        ),
        (
            "standard: the name in the caption's run, a district left out",
            page(vec![
                run(19.1, 344.8, "Penerima : B**i S**o"),
                run(19.4, 335.9, "DKI JAKARTA, KOTA ADMINISTRASI CONTOH,"),
            ]),
            area(Some("DKI JAKARTA"), Some("KOTA ADMINISTRASI CONTOH"), None),
        ),
        (
            "boxed: the name first, then the area and the street",
            page(vec![
                run(11.4, 221.8, "Penerima"),
                run(11.4, 210.4, "B**i S**o"),
                run(
                    11.4,
                    201.1,
                    "Jawa Barat, Kab. Contoh, Contoh Tengah, Jl. Contoh n",
                ),
                run(11.4, 193.9, "o 1 RT 2"),
            ]),
            area(
                Some("Jawa Barat"),
                Some("Kab. Contoh"),
                Some("Contoh Tengah"),
            ),
        ),
        (
            "instant: the end of the address, broken inside a word and between words",
            page(vec![
                run(14.8, 349.8, "Penerima :"),
                run(14.8, 326.9, "Alamat :"),
                run(14.7, 315.0, "Jl. Contoh No. 1, Desa Contoh, Contoh Teng"),
                run(14.7, 303.5, "ah, Kota Administrasi"),
                run(14.7, 292.0, "Contoh Barat, Jawa Ba"),
                run(14.7, 280.5, "rat"),
                run(160.3, 252.2, "(+62)88*******13Pengirim:"),
            ]),
            area(
                Some("Jawa Barat"),
                Some("Kota Administrasi Contoh Barat"),
                Some("Contoh Tengah"),
            ),
        ),
        (
            "Shopee-style: the province broken inside a word, the city line of three",
            page(vec![
                run(12.7, 316.5, "Penerima:"),
                run(50.9, 315.8, "Budi"),
                run(166.8, 315.8, "Pengirim:"),
                run(196.5, 301.6, "6280000000000"),
                run(
                    14.1,
                    291.1,
                    "Jl. Contoh No. 1, CONTOH TENGAH, KAB. CONTOH, JA",
                ),
                run(196.5, 290.4, "KOTA LAIN"),
                run(14.1, 283.6, "WA BARAT"),
                run(183.9, 266.4, "Jl Lain"),
                run(15.0, 250.7, "KAB. CONTOH"),
                run(117.0, 250.0, "CONTOH TENGAH"),
                run(208.3, 250.0, "DESA CONTOH"),
                run(12.7, 235.0, "Berat:"),
            ]),
            area(
                Some("JAWA BARAT"),
                Some("KAB. CONTOH"),
                Some("CONTOH TENGAH"),
            ),
        ),
        (
            "Shopee-style: broken between words, a city line with no district",
            page(vec![
                run(8.5, 310.6, "Penerima:"),
                run(142.8, 309.8, "Pengirim:"),
                run(
                    8.5,
                    296.3,
                    "Jl. Contoh 2, KEBON CONTOH, KOTA CONTOH BARAT, DKI",
                ),
                run(8.5, 288.1, "JAKARTA"),
                run(27.5, 257.4, "KOTA CONTOH BARAT"),
                run(52.3, 241.7, "CASHLESS"),
                run(8.5, 229.0, "Berat:"),
            ]),
            area(Some("DKI JAKARTA"), Some("KOTA CONTOH BARAT"), None),
        ),
        (
            "Shopee-style: a tag, a justified line word by word, the city line close under",
            page(vec![
                run(12.7, 316.5, "Penerima:"),
                run(166.8, 315.8, "Pengirim:"),
                run(16.0, 303.1, "OFFICE"),
                run(14.1, 291.1, "Jl. Contoh No. 3 (dekat toko),"),
                run(196.5, 290.4, "KOTA LAIN"),
                run(14.1, 283.6, "CONTOH TENGAH,"),
                run(63.6, 283.6, "KOTA"),
                run(85.5, 283.6, "CONTOH,"),
                run(129.3, 283.6, "JAWA"),
                run(151.6, 283.6, "TIMUR"),
                run(6.0, 276.0, "SPXID00000000001A"),
                run(25.1, 273.1, "KOTA CONTOH"),
                run(121.4, 272.4, "CONTOH TENGAH"),
                run(12.7, 257.4, "Berat:"),
            ]),
            area(
                Some("JAWA TIMUR"),
                Some("KOTA CONTOH"),
                Some("CONTOH TENGAH"),
            ),
        ),
        (
            "no receiver's caption",
            page(vec![run(19.0, 344.1, "Pengirim :")]),
            Area::default(),
        ),
    ];
    for (case, page, want) in cases {
        assert_eq!(find_area(&page), want, "{case}");
    }
}
