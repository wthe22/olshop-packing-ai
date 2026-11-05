// **Print**'s preview over the app's own window: an output's bytes in a hidden frame, whose
// `print()` shows the web view's preview of the PDF itself, with Chrome's options. Printing the
// window would print the app, and a PDF opened as a page of its own prints as one blank sheet. The
// preview does not say whether Print or Cancel was pressed.

// The frame of the last print; it stays until the next, so a preview still open keeps its PDF.
let shown: HTMLIFrameElement | null = null;

// Resolves once the preview has been asked to open.
export function printPdf(bytes: ArrayBuffer): Promise<void> {
  if (shown) {
    URL.revokeObjectURL(shown.src);
    shown.remove();
  }
  const frame = document.createElement("iframe");
  frame.style.cssText = "position: fixed; width: 0; height: 0; border: 0; visibility: hidden";
  frame.src = URL.createObjectURL(new Blob([bytes], { type: "application/pdf" }));
  shown = frame;
  return new Promise((resolve, reject) => {
    frame.onload = () => {
      try {
        frame.contentWindow?.print();
        resolve();
      } catch (e) {
        reject(e);
      }
    };
    document.body.append(frame);
  });
}
