# Plan

Only the unfinished work. A finished task is deleted from this folder in the commit that finishes it; a
finished phase moves to `archive/`. Phase and task numbers live only here. The code rules apply to every
task: a task report states *size gate passed, no new lint suppression, coverage not lower*.

## Status (2026-11-27)

- **Next (2026-11-27) — the journal goes, [journal-removal.md](journal-removal.md) J1 to J6.** The owner's
  decision [D-APP-39](../../software-design/decisions/app.md): `day.toml`, `orders.csv` and
  `order-contents.toml` are the truth of a date, with no history (no *amended*, no record of reverts or
  marks); Print sets `press_print_time`, the tick `is_printed`. The design is written; the code still has the
  journal. It replaces the data rework's T3 and T4 (T1 and T2 are in the code). The owner's answers to Q-APP-24 to
  Q-APP-26 are [D-APP-40, D-APP-41](../../software-design/decisions/date-files.md) (Read labels again for a
  changed or missing file and for Check data) and [D-LABEL-29](../../software-design/decisions/label.md) (no
  courier clue). A new session
  takes the first task left in journal-removal.md (J1). No build until J6.

- **Then — the batch screens, [batch-screens.md](batch-screens.md) U1 to U5**, after the journal removal. The
  owner's findings (2026-11-27): no examples in boxes, reading stays on New batch with each stop under its
  file, the sort rule chosen on the Plan screen with *Custom* for a plan changed for one batch, **Add Group**,
  and a Summary screen read in the app ([D-UI-17 to D-UI-19](../../software-design/decisions/ui-screens.md),
  [D-RULES-7](../../software-design/decisions/rules.md)). Open for the owner: archive mode,
  [Q-APP-27](../../software-design/questions/app.md); answers go under the question.

- **Handoff of 2026-11-27 (latest) — v0.8.0-alpha.11: Update lists its moves and copies what it changes.**
  The owner's rule ([D-APP-36](../../software-design/decisions/versions.md)): a file only moved or renamed is
  not backed up, the move is written down so a script can undo it; a file whose content changes is copied.
  - **Update** now starts `update v0.8.0-alpha.11\` in each date it changes: `update.toml` lists every move (written
    before the move) and the folder keeps copies of the journal, `day.toml`/`session.toml`, `orders.csv` and
    `order-contents.toml`. A date of v0.7 is made flat by moving its files, not copying them: no
    `v0.7 backup\` any more. A file in the way goes to `replaced\` of the update folder.
  - **`tools/update-backup.py revert "<data folder>"`** (`--dry-run` first) undoes it date by date; a date
    saved to after the Update is left and named. Tried on a made-up v0.7 date: after Update and revert every
    file had its old name and bytes again.
  - **Version numbers** are semver pre-releases: this build is `0.8.0-alpha.11`; app versions compare in
    semver order. The live data is still v0.7, so nothing reads the test builds' old numbers specially. The
    script needs no version when the data folder holds Updates of only one.
  - **`orders.csv`**: an older copy now also reads the version in its last header and leaves a newer one;
    a column added by hand is still written over by the next save (the guide says to edit a copy).
  - **To check**: on a copy of a v0.7 date (or a date with `session.toml`), choose Update, look into
    `update v0.8.0-alpha.11\`, then run the script with `--dry-run` and without.
  - **Where to answer**: [reviews/app.md](../../reviews/app.md) as `R-APP-20`.

- **Handoff of 2026-11-27 — v0.8.0-alpha.8: nothing is lost; what is not known is named quietly.**
  The owner's rule ([D-APP-34](../../software-design/decisions/versions.md)): nothing is lost but by an action
  meant to remove it, and what the program does not know in data of its own version is named without
  getting in the way.
  - An account edited on the Accounts screen or by the shop prompt now keeps its other keys and the owner's
    comments in `config.toml` (it was rebuilt from its name and shops); a label group already did. A key
    keeps the comment above it and the one after its value when its value changes.
  - **Settings › Data › Kept, not read** (only when there is one): each key of `packing-app.toml`,
    `parser.toml` or `config.toml` this version does not know, with file and line, unless the file is of a
    newer version. A slip such as *enabeld = false* shows there.
  - **Day screen, muted**: a record type or field of the journal this version does not know, once per type or
    field with its lines; one note for a newer copy's. The owner's data (two folders, four dates) shows none.
  - **To check**: add `colour = "red"` under an account in `config.toml`, open Settings (the row names it),
    rename that account on the Accounts screen, and see the key still in the file.
  - **Where to answer**: [reviews/app.md](../../reviews/app.md) as `R-APP-19`.

- **Handoff of 2026-11-27 — v0.8.0-alpha.7: a copy reads its own major version; `day.toml`.**
  The owner's answers to Q-APP-21 to 23 ([decisions/versions.md](../../software-design/decisions/versions.md)).
  - **D-APP-34**: a copy reads every `X.Y` of its own X and passes over what it does not know (a key, a
    field, a record type, a column); a higher X says *newer format, update this copy*. The journal stays
    `2.0`: a record type this copy does not know is passed over, with a note on the Day screen, and the date
    still takes commits. An older copy writes no view it would write short, and keeps unknown keys of
    `packing-app.toml`, `parser.toml` and `config.toml`. v0.7 still calls v0.8's `areas_filled` damage.
  - **D-APP-35**: `session.toml` is `day.toml` (**Update** writes it and deletes `session.toml`);
    `orders.csv` ends with the column `schema:2.1`, its cells empty.
  - **Changed for the owner**: a mistyped key in `config.toml` or `parser.toml` (*enabeld = false*) no longer
    stops the start; v0.8.0-alpha.8 names it in Settings.
  - **To check**: **Update** on a copy of the v0.7 data: each date gets `day.toml`, loses `session.toml`, and
    `orders.csv` opens in Excel with the empty *schema:2.1* column at the right.

- **Handoff of 2026-11-26 — v0.8.0-alpha.6: Search's cards under heads, one row each.** Built,
  checked and pushed; `target/portable/` is rebuilt. No dev task is left.
  - **R-UI-58** ([D-UI-16](../../software-design/decisions/ui.md),
    [ui/search.md](../../software-design/ui/search.md)): the heads *Order*, *Tracking*, *Saved*, *Label* are
    back over the cards, with **Select all** as their tick. A card's head is one row: the shop and Order ID,
    the courier and tracking ID, the batch's date and save time, the batch and group code; then the group's
    name (from the sort rule, without the count) and the page where the results are 72 rem wide or more,
    else under *GROUP* in the details. *Ship by* and *Weight* are in the details; the hash reads
    *SHA256: …*. Only the tick ticks; a click on a card opens and closes its details (a drag or a double
    click selects text and changes nothing; a click inside the open details changes nothing).
  - Search now sends each order's group name from its batch's sort rule (`group_name`); no stored data
    changes.
  - **To check**: Search on the owner's screen at 125 %: whether the group's name and page fit in the row
    there; a click on a card opens it; the ticks and **Select all** still choose the reprint.
  - **Where to answer**: findings into [reviews/app.md](../../reviews/app.md) as `R-APP-19`,
    [reviews/label.md](../../reviews/label.md) as `R-LABEL-20` (a weight or an area read wrong: the date,
    the order and what the label prints), [reviews/ui.md](../../reviews/ui.md) as `R-UI-59`. A new session
    turns each into a task and builds it.

- **Handoff of 2026-11-26 (late) — v0.8.0-alpha.5: Search shows each order as a card.** Built, checked and
  pushed.
  - **R-UI-57**, the owner's drawing ([D-UI-16](../../software-design/decisions/ui.md),
    [ui/search.md](../../software-design/ui/search.md)): one card per order, no head row. Its head is two
    lines on columns all cards share: the shop over the Order ID, the courier over the tracking ID, the
    batch's date and save time over the batch, *Ship by* over *Weight*; **Open** and *Reprinted* at the
    right. Under it the items as a table (quantity, name, variation, Seller SKU), the Qty Total at its foot
    when the order has several lines. The bar under the items opens *Address* (province, city, district, in
    the drawing's order), *Input file* (pages, short hash) and *Group* (code, name, the page in the group PDF).
  - **Kept although the drawing has no place for them** (the owner asked what was missing): the tick for
    Reprint; the *Reprinted* mark under **Open**; the order's page in its group PDF, in *Group*; the pickup
    code, only when the tracking ID is another; the account, only when the shop has another name. A long
    shop name ends at the Order ID's width, whole in its tooltip.
  - **To check**: Search on the owner's screen at 125 % over a few dates: the columns line up from card to
    card, long names wrap, and a narrower window puts the date and the weight under the shop and courier.

- **Handoff of 2026-11-26 (night) — v0.8.0-alpha.4: the weight; Search details without repeats; one schema
  step for v0.8.** Built, checked and pushed.
  - **R-LABEL-19**: each order records the parcel's weight in grams as its label prints it (`Weight :
    0.490 KG` on the TikTok labels, `Weight` `5 KG` on J&T Cargo's, `Berat: 450 gr` on the Shopee-style
    one), in the journal and as `weight_grams` in `orders.csv` ([D-LABEL-27](../../software-design/decisions/label.md)).
    **Update** reads it for the batches saved before, with the addresses (*Addresses and weights of 2
    batches, 61 orders*). On the samples every order reads one but the instant labels (Gojek, Grab) and 5
    IDX and Anteraja labels, which print none; nothing else the samples read changed.
  - **R-UI-55**: a Search row's arrow puts each item's Seller SKU under it and opens one line of facts:
    *Receiver* (district, city, province), *Weight*, *Ship by*, *Qty Total*, *Pickup code*, and *Account*
    only when the shop has another name; what the label does not print is left out; then the *From* line.
    The items table, the repeated account and *Courier named by* are gone (the clue only explains a courier
    named wrong; the journal and `orders.csv` keep it). Three designs were shown; A is built.
  - **R-UI-56**: Search opens again with its boxes as they were left, while the program runs; the dates start
    from the viewed day. **Reset** beside **Search** empties the boxes and sets the dates back.
  - **R-APP-18** ([D-APP-3](../../software-design/decisions/app.md)): `session.toml` is schema 2.1 for all of
    v0.8 (v0.7 wrote 2.0); a test copy updated by v0.8.0-alpha.2 or v0.8.0-alpha.3 (2.2) is asked about once more.
  - **Q-APP-20 answered** ([D-APP-33](../../software-design/decisions/app.md)): the theme and Settings ›
    **Change** still work in read-only; nothing in the data folder changes. The program already did so.
  - **Tried on a copy** of `2026-11-25` put back into the v0.7 layout: **Update** asked *Files of v0.7 moved
    in · Addresses and weights of 1 batch, 75 orders · the date's lists*; read-only changed no file; Update
    moved the files back, read the labels (*Reading labels: 2026-11-25, batch 1, page 3 of 75*) and wrote the
    lists: all 75 orders have an area and 72 a weight (the 3 others are Grab and Gojek labels, which print
    none); `session.toml` is 2.1; the next start did not ask again; `remove-v07-backup.py` removed the backup.
  - **The owner's steps for the v0.7 data**: as below (keep a copy, start, **Update**, check, then
    `remove-v07-backup.py --dry-run`); Search now shows the weight too.

- **Handoff of 2026-11-26 (later) — v0.8.0-alpha.3: Packing converts a v0.7 data folder itself; Search
  shows the whole order.** Built, checked and pushed.
  - **R-APP-17** (the owner's flow: the program converts, a script cleans up): a date folder of v0.7
    (with `input\` or `output\`) is one more line of **Update the data folder?** (*Files of v0.7 moved
    in*). **Update** moves its `input\`, `output\` and old summaries unchanged into `v0.7 backup\` of the
    date, copies each file up under its v0.8 name and checks its bytes, then reads the addresses and writes
    the lists (*Moving files: 2026-11-25, file 3 of 12*). **Open read-only** changes nothing, as before.
    `tools/flatten-data.py` is now `tools/remove-v07-backup.py`: it only deletes each backup whose files
    are all in the date folder (`--dry-run` lists first) ([D-APP-32](../../software-design/decisions/app.md)).
  - **R-UI-54**: a Search row's arrow opens every field of the order: account, province, city, district,
    ship-by date, Qty Total, the pickup code of an instant label, what named the courier; a table of its
    items with variation and Seller SKU; and the line it came from. The row's items show their variation.
  - **Tried on a copy** of `2026-11-25` put back into the v0.7 layout: the start asked (files of v0.7
    moved in, addresses of 1 batch, 75 orders); read-only changed no file and the Day screen named the
    date as v0.7; **Update** showed the files being moved and the pages being read, put all 11 PDFs back
    under their names with their bytes, filled the addresses of all 75 orders and wrote `session.toml`
    2.2 (2.1 since v0.8.0-alpha.4); the next start did not ask again; `remove-v07-backup.py --dry-run` named the
    backup and the run without it removed it.
  - **The owner's steps for the v0.7 data**: keep a copy of the data folder. Close every copy of Packing,
    put the new program in place, start it: it asks. **Update**, then look at a few dates (Open opens the
    group PDFs; Search shows the addresses). Then close Packing and run
    `python tools\remove-v07-backup.py "<data folder>" --dry-run`, then without `--dry-run`.

- **Handoff of 2026-11-26 — v0.8.0-alpha.2: the receiver's area, migrations that ask, read-only.** Built,
  checked and pushed; `target/portable/` is rebuilt.
  - **R-LABEL-17**: each order records the receiver's province, city and district as the label prints them
    (the standard, boxed, instant and Shopee-style layouts; empty where a layout prints none), in the
    journal and as three columns of `orders.csv` after `shop` ([D-LABEL-26](../../software-design/decisions/label.md)).
    All 744 sample orders read all three.
  - **R-LABEL-18**: an instant order saved before its pickup code became its tracking ID now shows the
    pickup code as its tracking ID; no file is changed.
  - **R-APP-16** ([D-APP-33](../../software-design/decisions/app.md)): at its start the program asks
    **Update the data folder?** before any date opens, listing each date and what changes (the addresses of
    its batches saved before, its lists). **Update** reads them from the label PDFs in the date folders,
    one line per date and the page being read, then **Continue**. **Open read-only** changes nothing in the
    data folder: the title bar says *Read-only*, and no button that writes is shown. Choosing another data
    folder asks the same.
  - **Tried on a copy** of `2026-11-25`: the start asked (one date, 1 batch, 75 orders); read-only changed
    no file of the copy and showed no button that writes; **Update** showed the page being read and a line
    for the date, filled the province, city and district of all 75 orders, wrote `session.toml` 2.2, and
    the next start did not ask again.
  - **What to try**: keep a copy of the data folder first. Start the new program: it asks. Choose **Open
    read-only**, look around (no file of the data folder changes), close it, start again and choose
    **Update**. Then a date's `orders.csv` shows the three columns.

- **Handoff of 2026-11-25 (evening) — v0.8.0-alpha: one flat folder per date; the split and Sameday labels
  read.** Built, checked and pushed; `target/portable/` is rebuilt.
  - **The conversion of the data folder** is the program's own since v0.8.0-alpha.3 (above); no script runs first.
  - **R-APP-12 to R-APP-15** ([D-APP-32](../../software-design/decisions/app.md)): every file of a date sits
    in its date folder; what a save writes waits in `.partial\` of the date folder until its commit.
    Inputs `2 Input 03 0a1b2c3d4e5f.pdf` (the batch whose record names them; bytes a second batch uses are
    stored again under its number), `2 Summary.pdf`, `2 Summary - Toko Alfa.pdf`, `Day Summary.pdf`,
    `Day Summary - Toko Alfa.pdf`. A summary the journal recorded as `1 summary.pdf` keeps that name.
  - **The split label** (`samples/2026-11-25 split.pdf`): a J&T label with no slip, and its slip on the
    next page, which prints the label's tracking ID again. The slip page now continues the label: one order,
    two pages, its slip lines, Qty Total and deadline ([D-LABEL-12](../../software-design/decisions/label.md)).
  - **R-LABEL-16**: an instant label's pickup code (`Kode Pengambilan`) is its tracking ID
    ([D-LABEL-24](../../software-design/decisions/label.md)); the courier is named as before. The Shopee-style
    instant code (`12AB`) is taken too, the owner's answer.
  - **The owner may test now**: after the conversion, open an old date: no missing files, Open opens
    each group PDF. Save a new batch: its inputs, group PDFs and `N Summary.pdf` sit in the date folder.
    Read `2026-11-25 split.pdf`: one order. Read `2026-11-24 sameday.pdf`: its two Grab orders show
    their pickup codes as tracking IDs.
  - **The regular SPX label** is named SPX by its `REGULAR SERVICE` badge, the owner's answer to Q-LABEL-14
    ([D-LABEL-25](../../software-design/decisions/label.md)). Read `2026-11-23 spx regular.pdf`: courier SPX,
    no logo asked about.
  - **Still the owner's step from 2026-11-25**: in the data folder's `parser.toml`, the `spx` classifier's
    `tracking_id = 'SPXID\d{11}[A-Z]'` becomes `'SPXID\d{11}[A-Z]|CM\d{11}'`, so the regular SPX label
    keeps its tracking ID; and its `logos` gain `"c622a9c866f1abf4"`, the badge (or answer **SPX** when the
    logo prompt shows the badge, once the form is changed). A new data folder has both already.
  - **Dead code removed**: the old reading path (`couriers.toml`, its courier table and deduction, the
    pages → orders step and its warnings), never used since copies; and one unused PDF reader function.

- **Handoff of 2026-11-25 — the source PDF's short hash in Search.** Built and pushed:
  [R-UI-53](../../reviews/ui.md): an expanded search row shows the source PDF's short hash after its file name
  (the input record's hash, [search](../../software-design/ui/search.md)). The owner may test: Search a saved
  order, open its chevron.

- **Handoff of 2026-11-23 (later) — a revert saves the label PDFs, not the outputs.** The owner's request,
  built and pushed: **Save** on a revert copies the batch's input files of `{date}\input\` (its label PDFs)
  into the new folder `{date} batch {n} reverted {hh.mm}`; the group PDFs, summary PDF and account summaries
  are no longer copied ([D-APP-31](../../software-design/decisions/app-features.md)). The question and the
  green line say *label PDFs*. `target/portable/` is rebuilt.
  - **The program is no longer in a subfolder**: `target/portable/Packing.exe` and `pdfium.dll` sit directly
    in `target/portable/` (was `target/portable/Packing/`); `tools/make-portable.sh` writes there.
  - **The owner may test now**: Revert on the newest batch, **Save**, pick a folder: the folder holds the
    batch's `…_NN_<hash>.pdf` files and nothing else; the input files stay in `input\`.
  - **Where to answer**: findings into [reviews/app.md](../../reviews/app.md) as the next `R-APP-<n>`. A new
    session turns each into a task and builds it.

- **Handoff of 2026-11-23 — v0.7.0: input names by date and number, the date on the batch screens, a revert
  that keeps its PDFs.** The
  owner's words in chat, built and pushed:
  - **R-APP-10**: an input in `{date}\input\` is `{date}_{pdf}_{short hash}.pdf`, e.g.
    `2026-11-23_03_61459e8a8203.pdf`: the date, its number on the date in two digits (so every list sorts
    it), the first 12 characters of its SHA-256 ([D-APP-10](../../software-design/decisions/app.md)). The
    journal is unchanged; the name follows from it.
  - **R-LABEL-15**: the short hash in the summary PDF and the day summary is set in Consolas.
  - **R-UI-48**: on a date that is not today, **+ New batch** is muted and asks *Add a batch to `<date>`?*
    ([D-UI-15](../../software-design/decisions/ui.md)). **R-UI-49**: New batch, Repeated orders and Plan
    show the open date over their bar like the Day screen's title, *today* or *not today* in amber.
  - **R-UI-50**: Search's first box is *Order ID contains*. A scanned tracking-ID barcode now needs the box
    switched to *Tracking ID*.
  - **R-APP-11**: **Revert** asks *Undo batch 2?* with **Save** and **Discard**. **Save** opens the
    Windows folder picker (the owner's answer B to Q-APP-19; closing it reverts nothing); the batch's group
    PDFs, summary PDF and account summaries are copied into a new folder `2026-10-07 batch 2 reverted 14.05`
    there, and the green line names it ([D-APP-31](../../software-design/decisions/app-features.md)).
  - **R-UI-51**: a button is one short word for what it does and never names its target: *Add*, *Save*,
    *Discard*, *Redo*, *Delete*, *Reprint*; each question's line says plainly what will happen
    ([ui/README.md](../../software-design/ui/README.md)).
  - **R-UI-52**: the account question (was *Which account?*, which could get stuck with no account to
    choose) is one list per row: *Which account are these orders for?*, a row per new shop name (shown as
    printed, no longer editable) and per file with labels that show no shop name. Every list offers the
    accounts, a new account another row names (*New: Toko Gamma*) and **New account**; a file's row starts
    on its other orders' account ([new-batch.md](../../software-design/ui/new-batch.md#the-shop-prompt)).
    The portable build and the guide pictures are not redone yet: another session builds them.
  - **Where to answer**: findings into [reviews/ui.md](../../reviews/ui.md) as `R-UI-53`,
    [reviews/app.md](../../reviews/app.md) as `R-APP-12`, [reviews/label.md](../../reviews/label.md) as
    `R-LABEL-16`. A new session turns each into a task and builds it.

- **Handoff of 2026-11-13 — the config files are three: `packing-app.toml`, `parser.toml`, `config.toml`.** The
  owner's layout ([D-APP-30](../../software-design/decisions/app-features.md)), built and pushed:
  - `packing-app.toml` beside `Packing.exe` holds the data folder and the theme. In the data folder,
    `parser.toml` holds the couriers and their clues, `config.toml` the accounts and the sort rules
    ([rules files](../../software-design/rules-file/README.md)).
  - **No conversion is built.** The program reads only the three new files; a data folder without
    `parser.toml` and `config.toml` is given the bundled copies. The owner moves the former content by
    hand: the data folder and theme into `packing-app.toml`; the couriers, classifiers and `not_courier`
    of `packing-workflows.toml` into `parser.toml`; its sort rules and the `[[accounts]]` of
    `packing-settings.toml` into `config.toml`.
  - **What changes for the owner**: the accounts, the sort rules and the learned logos are in the data
    folder now, so they follow it to another PC and are shared by copies on one data folder; choosing another
    data folder in Settings switches them too (a folder without the files is given the bundled copies).
  - **To try** in `target/portable/`: start it; the accounts and sort rules are as before; open
    `label-data\config.toml` and `parser.toml`; add an account on the Accounts
    screen and see it in `config.toml`; answer a logo and see it in `parser.toml`.
  - **Where to answer**: findings into [reviews/ui.md](../../reviews/ui.md) as the next `R-UI-<n>`.
  - Version 0.6.0. The sample data in `target/portable/` was cleared; the bundled courier code of J&T Express is `jnt-express`.

- **Handoff of 2026-11-12 (second) — the owner's eight screen findings and the Grab Express label;
  version 0.5.0 rebuilt.** The owner's words in chat, built and pushed:
  - **Search** (R-UI-40..42): a row reads *shop* then the Order ID, both at the row's text size; the
    *Label* column is *Wed, 11 Nov Batch 2* over *the group's name page 3*; the details are one line,
    *From [input.pdf] p. 3, saved at 09:12 into group 2-A* ([search](../../software-design/ui/search.md)).
  - **Dates panel** (R-UI-43): only the date and its orders; `· not printed yet` is gone, code included.
  - **Accounts** (R-UI-44, R-UI-45): **+ New account** asks the name in a dialog; no text of the app
    ends in `…` any more (**Rename**, *Loading*, *Saving*, *Reading labels: page 3 of 10*).
  - **Day** (R-UI-46, R-UI-47): a batch with several accounts has **Summary PDF ▾** as a menu like **Day
    summary** (*All accounts* opens the batch's file; an account writes its summary); the batch head says
    *2 accounts* instead of the names and counts. The owner wrote "3 shops"; the app's word for what that
    line counted is *account*, so it says *accounts*. If the owner wants the shops counted (a shop's
    TikTok and Shopee spellings would count twice), that is the next `R-UI-48`.
  - **Grab Express on Shopee** (R-LABEL-14, `samples/2026-11-11 grab-express.pdf`): read as Grab, its
    `No. Pesanan:` and its pickup code (two digits + two letters) kept
    ([business-process 02](../../business-process/02-data-sources.md)).
  - **The owner's step first**: the program reads the owner's own `parser.toml` in the data folder (a copy of
    the former `packing-workflows.toml`, see the handoff above), never the bundled copy (D-APP-3). In its
    `courier_code = "grab"` entry the owner changes
    `instant_tags = ["Sameday"]` to `instant_tags = ["Sameday", "Same-day"]`, saves and starts the program
    again. Without that line the order reads with no courier (a warning; it is still sorted and saved).
  - **Open question**: **Q-LABEL-13** ([questions/label.md](../../software-design/questions/label.md)):
    this label breaks a product name inside a word, so its lines are joined with nothing between (default
    A: *Samping*, *SEPATU-HITAM*, but a break on a space glues two words, `Standar |Penyangga`).
  - **The owner may test now** in `target/portable/`: Search a saved order and open its details;
    New account; a batch of two accounts on the Day screen; read the 2026-11-11 file after the step above.
  - **Where to answer**: a screen finding into [reviews/ui.md](../../reviews/ui.md) as `R-UI-48`, a label
    finding into [reviews/label.md](../../reviews/label.md) as `R-LABEL-15`, Q-LABEL-13 in its question.
    A new session turns each into a task and builds it.
- **Handoff of 2026-11-12 — v0.5.0: accounts replace workspaces. The owner's review is next.** Built,
  checked and pushed; no task is open. The design is in [software-design/](../../software-design/README.md)
  (start with [accounts](../../software-design/app/accounts.md)); the finished plan is in the archive
  (`v0.5-accounts.md`).
  - **What to try** in `target/portable/` (version 0.5.0), with a copy of real downloads of two
    shops: New batch reads both; the first read asks *Which account?* for each shop; the Day screen names
    each batch's accounts, with a summary button per account and **Day summary** as a menu; **Accounts** in
    the title bar shows each account's shops and dates; Search finds an order by its shop (the *Shop* and
    *Account* fields) and shows the shop under each Order ID. The [user guide](../../user-guide/README.md) is
    current, with pictures of v0.5.0.
  - **Open question**: **Q-APP-18** (the saved orders of a renamed account,
    [questions/app.md](../../software-design/questions/app.md)), built with its default A: orders saved
    before keep the old name.
  - **Checks**: the full check is green (size gate passed, no new lint suppression, coverage not lower);
    an independent review found no journal or data bug; its migration-script findings are fixed.
  - **Where to answer**: findings into [reviews/](../../reviews/README.md) as the next `R-UI-<n>` or
    `R-APP-<n>`, one file per scope; an answer to Q-APP-18 in its question. A new session starts here,
    turns each finding into a task file in this folder and builds it.
  - Not part of this: files another session left uncommitted (`docs/development/README.md`,
    `docs/development/app.md`, `tools/dev.bat`).
- **Handoff of 2026-11-09 — the panel marks a workspace with unprinted groups today; version 0.4.2.** The
  owner's words in chat, built and pushed:
  - The workspace list of the panel shows **`· not printed yet`** after a workspace whose live batches of
    today have a group PDF not marked **Printed** ([panel](../../software-design/ui/panel.md)). The mark is
    read from each workspace's journal when the panel is read, so it clears on the next read after a tick.
  - Not in this change: the dates list of the open workspace has no mark, and a summary PDF does not count.
  - `target/portable/` is rebuilt. **The owner may test now**: tick Printed on every group PDF of
    today in one workspace, then open the workspace list; the mark goes when no group is left. A finding goes
    into [reviews/ui.md](../../reviews/ui.md) as the next `R-UI-<n>`.

- **Handoff of 2026-11-06 — a workspace's shops; Print in the app; version 0.4.0.** The owner's words in
  chat, built and pushed:
  - **D-APP-25**: a workspace lists its shops (Settings › Workspaces › **Shops**, kept in
    `{workspace}/settings.toml`). A read whose labels show another sender, or none, asks first:
    **Back**, or **Read anyway**. Names are compared whole, spaces and case ignored, so the Shopee
    spelling is listed beside the TikTok one
    (v0.4; replaced by [accounts](../../software-design/app/accounts.md) in v0.5). A workspace with no shop
    listed asks nothing.
  - **D-APP-26**: **Print** on a group PDF, a summary PDF or a reprint opens the print preview Chrome shows
    over the app's window. A group PDF is then marked *Sent hh:mm*; its ✕ takes the mark back when the
    print was cancelled or failed. The **Printed** tick is the second check and can be ticked with or
    without *Sent* ([Day](../../software-design/ui/day.md)). The preview does not tell the app whether
    Print or Cancel was pressed.
  - **Update every copy together**: the journal now holds `print_sent` records, and an older copy of the
    program reads such a date as damaged ([records](../../software-design/data/records.md)).
  - **The owner may test now** with `target/portable.exe`: list the shops of each
    workspace, then read one day's real downloads, a file of another shop among them. Print a group PDF, cancel once (✕ the mark), print again, tick
    Printed. A finding goes into [reviews/app.md](../../reviews/app.md) as R-APP-10, a screen finding into
    [reviews/ui.md](../../reviews/ui.md) as R-UI-39, a sender read wrong into
    [reviews/label.md](../../reviews/label.md) as R-LABEL-13.

- **Handoff of 2026-11-02 — several copies on one data folder; version 0.3.3.** The owner's words in
  chat, built and pushed:
  - **R-APP-9**: a second copy of the program no longer stops at *Packing is already open*. The data
    folder's lock is held only while a copy writes; another copy's write waits for it, a minute at most
    ([D-APP-9](../../software-design/decisions/app.md), [data › One writer](../../software-design/data/README.md#one-writer)).
  - Each copy looks every 3 seconds for what the others wrote: the Day screen, the panel and Search
    follow. A write reads the date's journal again first; a save whose orders another copy saved after
    the labels were read stops with *read the labels again*; **Revert** and **Amend** refuse a batch that
    is no longer the newest; a workspace another copy renamed or deleted is followed
    ([program › Several copies](../../software-design/app/program.md#several-copies)).
  - Version **0.3.3**. The commit also holds the earlier session's 0.3.2 work that was left uncommitted
    (the whole version shown, *About*'s build time, the buttons without `…`).
  - **The owner may test now**: start `target/portable.exe` twice, or on two PCs with one
    shared data folder. Save a batch in one: the other shows it within a few seconds. Tick Printed in
    both, save in both at once, rename the workspace in one. A finding goes into
    [reviews/app.md](../../reviews/app.md) as R-APP-10.
  - Not covered: `packing-workflows.toml` is beside the program, not in the data folder. Copies started
    from one program folder share it, and the last save of a sort rule wins.

- **Handoff of 2026-11-02 — Shopee orders: the tracking ID's letter, `total_quantity` on every slip.**
  The owner's words in chat, built and pushed:
  - **R-LABEL-12**: an SPX tracking ID is `SPXID` + 11 digits + a capital letter, and the letter was
    dropped. The bundled form is `SPXID\d{11}[A-Z]`; the `No. Resi` layout's `Batas Kirim` date is read
    too ([labels/courier](../../software-design/labels/courier.md)). The owner's files are in `samples/`
    as `2026-11-02 spx eco.pdf` and `2026-11-02 spx std.pdf`.
  - **R-RULES-7**: a filter's `total_quantity` is the units of the order's lines on every slip. It was
    the printed `Qty Total`, which the Shopee-style slip does not print, so `total_quantity = 2` never
    matched there and the order fell to the next sepatu group
    ([conditions](../../software-design/rules-file/conditions.md)).
  - **The owner's step first**: the program reads the owner's own `packing-workflows.toml` beside
    `Packing.exe`, never the bundled copy, and does not rewrite it (D-APP-3). In its
    `courier_code = "spx"` entry the owner changes `tracking_id = 'SPXID\d{11}'` to
    `tracking_id = 'SPXID\d{11}[A-Z]'`, saves and starts the program again. Without that line the
    tracking ID still loses its letter; the `total_quantity` fix needs no change to the file.
  - **The owner may test now**: read the two 2026-11-02 Shopee PDFs with `main-v2.0`: the sepatu
    orders of 2 go to *Sepatu Hitam x2*, and each tracking ID ends in its letter. A batch already saved
    keeps what it read until it is amended. A finding goes into [reviews/label.md](../../reviews/label.md)
    or [reviews/rules.md](../../reviews/rules.md) as R-LABEL-13 or R-RULES-8.
  - `target/portable/` is built from the working tree, which also holds another session's
    uncommitted work (as the build of 2026-11-01 did); that work stays uncommitted.

- **Handoff of 2026-10-30 — Search's dates and rows, the date as the Day screen's title.** The owner's
  words in chat, built and pushed:
  - **R-UI-37**: no *This day* anywhere. Search's Dates are *Viewed day* (the date the Day screen shows,
    the default; not listed when it is today), *Today*, *Yesterday* and the rest
    ([D-UI-3](../../software-design/decisions/ui.md)). The Day screen's title is the date itself with a
    *today* pill; the title bar names only the workspace ([ui/day](../../software-design/ui/day.md)).
  - **R-UI-38**: a Search row is *Order* (the Order ID over the courier and tracking ID), *Items*, *Label*
    (the date and group code over the group's name and page); its details show only the batch's save
    time and the input PDF, its file name boxed with a file icon, no courier clue
    ([D-UI-12](../../software-design/decisions/ui.md)).
  - **The owner may test now**: Search on both screens, a row's arrow, the Dates list on an older date
    and on today. A finding goes into [reviews/ui.md](../../reviews/ui.md) as R-UI-39.

- **Handoff of 2026-10-30 — the sort rules: the rule used last, a rule switched off, a new page.** The
  owner's words in chat, built and pushed:
  - **R-RULES-5**: New batch preselects the rule of the workspace's newest batch **on whichever date**, so
    a new day starts on the rule used last; **Back** from the Plan screen keeps the rule just read
    ([D-RULES-1](../../software-design/decisions/rules.md)). Nothing new is stored: the journals say it.
  - **R-RULES-6**: a sort rule is **On** or **Off** (`enabled = false` in `packing-workflows.toml`); New
    batch lists only the rules that are on; the only rule that is on cannot be switched off or deleted
    ([D-RULES-5](../../software-design/decisions/rules.md)).
  - **The Sort rules page** in the look of Settings and the Plan screen
    ([ui/sort-rules](../../software-design/ui/sort-rules.md)): a tab per rule (*last used*, *off*), the
    rule as one card — its head the switch, **Rename…** in place and **Delete rule**, a row per group with
    its code as the blue chip, **Boxes | Text**, the count under *In batch n* and icon buttons — and a foot
    that stays in view with *Unsaved changes*, **Cancel** and **Save**; another tab asks before it drops
    unsaved changes. The doubled *no filter: takes the rest* is one tick.
  - **The owner may test now**: [R8-checklist.md](R8-checklist.md) item 10. A finding goes into
    [reviews/rules.md](../../reviews/rules.md) or [reviews/ui.md](../../reviews/ui.md) as the next
    `R-<SCOPE>-n` (R-RULES-7, R-UI-36).

- **Handoff of 2026-10-29 — the owner tests round eight; then the work is R9.** Round eight of
  the owner's review, from the owner's words in chat, is built and pushed:
  - **Search** shows one compact **row** per order again: the Order ID over the tracking ID; the items,
    with the rest of the row; the group PDF's code, name and **page** over the courier and the date;
    **Open**, the trace arrow and the *Reprinted* mark ([D-UI-12](../../software-design/decisions/ui.md)).
    The ID and group PDF columns are only as wide as their longest entry. In a window much less than full
    width the items take a line of their own under the IDs and the dates and **Search** a line under the
    filter; the note of a reprint just written sits in the reprint bar's line.
  - **The day summary** lists the day's order contents with the most orders first (the pick pages keep
    their order) and, after the couriers, each batch's input PDFs with their pages and SHA-256 and the
    batch's ledger, as a batch's summary PDF does
    ([D-LABEL-23](../../software-design/decisions/summaries.md)).
  - In a window under 56 rem the title bar's **Search**, **Sort rules** and **Settings** keep their icons
    only, so the bar no longer wraps ([ui/README](../../software-design/ui/README.md#window-frame)).
  - Round seven's look ([D-UI-11](../../software-design/decisions/ui.md)) and *Tracking ID contains* stay.
  - **Found while checking, not fixed**: the PDF viewer of WebView2 (Edge's) draws the `·` of the summary
    PDFs and the day summary as `¥`; the text layer reads `·`. `·` is drawn with the non-CID font
    (`page/fonts.rs`), and the verifier reads the header's `·` line as text, so a fix touches both.
  - Checked on screenshots of the built program at 1536 × 800, 1093 × 580 (the owner's two screens at
    125 %) and at 960, 768 and 600 points wide, in both themes, with long made-up product names.
  - `target/portable/` is rebuilt. **The owner may test now**: Search on the small screen and in a
    window less than full width (a long product name, ticking rows, **Select all**, **Reprint**), and
    **Day summary**'s first and last pages; then [R8-checklist.md](R8-checklist.md) items 9 and 11 if not
    tried.
  - No question is open.
  - A finding goes into [reviews/ui.md](../../reviews/ui.md), [reviews/label.md](../../reviews/label.md),
    [reviews/rules.md](../../reviews/rules.md) or [reviews/app.md](../../reviews/app.md) as the next
    `R-<SCOPE>-n` (R-UI-36, R-LABEL-12, R-RULES-7, R-APP-9); the next session turns each into a task.
  - **Next without the owner:** **R9** (the full
    check, an independent review of the whole v0.2 diff, the coverage baseline, the handoff). An instant
    label's unknown tag is not asked about yet (W-9 of [workflow-audit.md](workflow-audit.md)).
  - The guide's pictures are the real program on the `testdata/` labels (`node tools/guide-pictures.mjs`
    drives a fresh copy over WebView2's DevTools port); the same run is a quick smoke test of the build.

- **The work is the v0.2 rework**: [v0.2-rework.md](v0.2-rework.md) holds the decisions agreed with the owner
  and the order of work. **R1 to R5 are done**: [software-design/](../../software-design/README.md)
  describes v0.2; the project has no Python; the label fixtures are written by `crates/fixtures` one
  character per text object, as the downloads are; today's reading of the owner's eleven sample files is
  pinned in `work/baseline-copies/` on this PC ([testing.md](../testing.md), *Local baseline*); `crates/store`
  holds the journal, the lock, the inputs, the views, the open-time checks and the workspaces; and
  `packing-engine` holds the v0.2 chain — `packing-workflows.toml`, copies with the courier clue,
  repeats and the copies left out, the plan with order contents, the `batch_saved` record — and the 2.32
  fix; `crates/verify` checks a batch's outputs by page digest with `lopdf` alone (every page of the
  samples that PDFium copies keeps its digest). **R6 is done**: the app and the command line save through
  `crates/batch`, and the v0.1 code is removed. **The program the owner runs is v0.3.3**: it
  saves through `crates/batch` and the journal into `{data_dir}/{workspace}/{date}/` and writes the summary
  PDF. A task gets its own file in this folder when it starts; until then its row in the order of work is
  the task.
- **The owner checked the v0.1 program** (2026-10-26): it works as before, including the full product name
  read without a space inside a word; the owner said to go ahead with R6. R6 is the first task that changes
  the program the owner runs: the app saves through the journal into `{data_dir}/{workspace}/{date}/` and no
  longer shows the v0.1 data (`labels\`, `state.json`), which stays on disk unread
  ([D-APP-17](../../software-design/decisions/app-features.md)).
- **Not blocked on the owner otherwise.** The owner may read the design at any time, starting with
  [the workflow](../../software-design/workflow/README.md) (each step against one page) and the points below;
  a finding goes into [reviews/app.md](../../reviews/app.md) under R-APP-2 and changes the design before the
  task that builds it.
- **The owner confirmed** (2026-10-25) the points R1 had to settle: the ship-by deadline is a date only
  (D-APP-21); orders are grouped by name, variation and quantity, the Seller SKU kept only for reference —
  names and SKUs can change during the day, and the order inside a group matters less than the grouping
  (D-APP-20); a `courier` filter compares the code; an amend uses the rule of the batch's name, else its
  snapshot (D-RULES-2); a continuation page must follow its label page (D-LABEL-12).
- **The owner chose *order content*** (2026-10-25) for the orders with the same items inside a group PDF
  ([D-LABEL-15](../../software-design/decisions/label.md)): the Seller SKU and the printed `Qty Total` moved
  from the content to the order, and the block number (`2-A-05`) is gone. The owner judges on the running
  screens (R8) whether the word fits; a finding goes into [reviews/ui.md](../../reviews/ui.md).
- **Kept for the rework**: [workflow-audit.md](workflow-audit.md), where the built program differs from the
  design; the v0.2 crates close its rows, and the program follows when R5 and R6 put them to use.
- The program folder `target/portable/` is rebuilt after each task; a rebuild replaces only
  `Packing.exe` and `pdfium.dll`, so the owner's settings, rules and data there stay.
  The owner's checklist for v0.2 is [R8-checklist.md](R8-checklist.md).

## Where to work

A **local** session on the owner's PC (Windows, Git Bash): it builds, runs `tools/make-portable.sh` and
commits. A cloud session cannot run commands on the PC; it can only read and write files there.

Outside Windows the summary PDF tests fail, because the summary writer loads its fonts from
`C:\Windows\Fonts` (ten tests of `packing-app`); that is not a regression.

## Reading for a new session

In this order.

1. [../../../AGENTS.md](../../../AGENTS.md) and [../README.md](../README.md): rules, commit and push, layout.
2. [v0.2-rework.md](v0.2-rework.md), then the task's own file when it has one.
3. [../../software-design/workflow/README.md](../../software-design/workflow/README.md) (the steps, the
   invariants I1–I12 and the verification matrix), then [data/](../../software-design/data/README.md) and the
   design files the task names.
4. [../code-rules.md](../code-rules.md) and [../testing.md](../testing.md).
5. [../../business-process/02-data-sources.md](../../business-process/02-data-sources.md) before touching
   any PDF code.
6. Code, after the documents: `crates/` and `app/`; then `git status` and `git log --oneline -5`.

`samples/` and `work/` are confidential: never commit them or copy real data from them.
