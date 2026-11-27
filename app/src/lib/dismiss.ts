// What closes a menu (panel.md): Escape, or a press anywhere outside the element that holds the
// menu and its button. An attachment (`{@attach open && dismiss(close)}`), so it listens only
// while the menu is open; choosing an item is the menu's own close.

export function dismiss(close: () => void) {
  return (node: HTMLElement) => {
    const onPointer = (event: PointerEvent) => {
      if (!node.contains(event.target as Node)) close();
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") close();
    };
    document.addEventListener("pointerdown", onPointer, true);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onPointer, true);
      document.removeEventListener("keydown", onKey);
    };
  };
}
