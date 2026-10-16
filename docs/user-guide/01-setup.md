# Setting up the program

The app **Packing** runs on a Windows PC. It takes the *Shipping label + Packing slip* PDFs of one
download from the seller centre, sorts the orders into picks, and writes one saved label PDF per
pick plus the packing list to print. No terminal is needed.

## Set up the program folder

The app is one folder, not an installed program.

1. Copy the folder `Packing` (the output of `tools/make-portable.sh`, found in
   `target/portable/Packing/`, or the folder you were given) to a place you can write to, for
   example `D:\Packing\`. **Do not** put it under `C:\Program Files` — Windows does not let the
   app write there.
2. The folder must contain both files:
   - `Packing.exe` — the program;
   - `pdfium.dll` — the PDF reader the program uses; it does not work without it.
3. Make a shortcut to `Packing.exe` on the Desktop or the taskbar: right-click `Packing.exe` →
   *Show more options* → *Send to* → *Desktop (create shortcut)*. Start the program from the
   shortcut.

The app keeps its data in the same folder (see [The first start](#the-first-start)). Copying the
whole folder to another PC moves the program and all its data along.

## The first start

![The Start prompt](images/01-start.png)

When the program starts and finds no data next to it, it says
**"No packing data next to the program (`D:\Packing\`)."** and offers two buttons:

- **Use this folder** — the app writes `couriers.toml` (the courier data) next to the program and
  keeps using this folder for every later start. It does not create a picks list: until you add
  picks, every order goes into one PDF set (see
  [Editing your categories](03-changes.md#editing-your-categories)). Choose this for a normal setup.
- **Choose a folder…** — pick another folder for the data. It is used only until you close the
  app; the next start asks again. Use this when the program folder cannot be written.

After that the **Day** screen opens.

## A new version of the program

When you get a new version, close the app, then copy the new `Packing.exe` and `pdfium.dll` over
the old ones in the program folder. Your data (`categories.toml`, `couriers.toml`, `settings.json`,
the `labels` folder) stays as it is.
