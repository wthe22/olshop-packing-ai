# Packing Checker — Documentation

Two tools for the daily packing work:

- a **PC script** that turns the orders CSV and shipping-label PDFs into grouped label files and
  an A4 packing list;
- an **Android app** that checks every packed parcel by scanning its label with the camera.

| Folder | Contains |
|---|---|
| [`business-process/`](business-process/) | How the packing work is done today, and the files it uses. Source of truth |
| [`software-design/`](software-design/) | How the tools should be built. Future, not implemented |

Rule: a statement that would still be true if the tools were never built belongs in
`business-process/`. A screen, field, rule or library belongs in `software-design/`.

`samples/` holds real platform files. It is confidential and is never committed.
