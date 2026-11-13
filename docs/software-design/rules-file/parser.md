# The parser file: `parser.toml`

How labels are read ([D-APP-30](../decisions/app-features.md)): the couriers, the clues that name them
and the header images that are not couriers. In the data folder; see [the two files](README.md) for how
it is written, edited and checked.

## Schema

```
Root {
  schema: String // X.Y version, starts at 1.0
  couriers: List<Courier>
  courier_classifier: List<CourierClassifier>
  not_courier: NotCourier
}

Courier {
  code: String // unique: lower-case letters, digits and -
  name: String // display name
}

CourierClassifier {
  courier_code: CourierCode // unique
  logos: List<String> // logo hash (XXH3-64 of the image's encoded bytes, 16 hex)
  shapes: List<String> // shape hash (XXH3-64 of the 64 x 64 appearance)
  tracking_id: Option<String> // tracking-ID regex, matched fully; none for a courier without one
  text: List<String> // printed text that stands in for the logo
  instant_tags: List<String> // service tags of an instant-service label (GKS)
}

NotCourier {
  logos: List<String> // hashes of header images that are not a courier, e.g. a channel strip
}
```

## Example

```toml
schema = "1.0"

[[couriers]]
code = "jnt"
name = "J&T Express"

[[courier_classifier]]
courier_code = "jnt"
logos = ["39b9b7b9f61d0e7f"]     # 975x377 px, the same asset in every sample file
tracking_id = 'J[XY]\d{10}'
text = ["jet.co.id"]

[[courier_classifier]]
courier_code = "jnt-cargo"
shapes = ["753ccbd14749befa"]    # masked wordmark: its bytes name nothing
tracking_id = '57\d{10}'
text = ["FastTrack"]

[[courier_classifier]]
courier_code = "gojek"
instant_tags = ["GKS"]

[not_courier]
logos = []
```

The bundled copy holds J&T Express (`jnt`), SiCepat (`sicepat`), J&T Cargo (`jnt-cargo`), IDX (`idx`), SPX
(`spx`), Anteraja (`anteraja`), Gojek (`gojek`, tag `GKS`) and Grab (`grab`, tags `Sameday` and `Same-day`) with the hashes,
forms and tags of [courier deduction](../labels/courier.md), and the buying channels' strips in
`not_courier`. A list field left out is empty.
