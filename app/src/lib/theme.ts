// The window's theme: the owner's choice (`packing-settings.toml`) made into the light or dark
// colour set of `app.css` through `data-theme` on <html>. *System* follows Windows, also when
// Windows changes while the program is open.

export type Theme = "system" | "light" | "dark";
export type Shade = "light" | "dark";

export const themes: Theme[] = ["light", "dark", "system"];

export function shadeOf(theme: Theme, windowsDark: boolean): Shade {
  if (theme === "system") return windowsDark ? "dark" : "light";
  return theme;
}

const windowsDark = () => window.matchMedia("(prefers-color-scheme: dark)");
let following: Theme = "system";
let listening = false;

// Show `theme` now and keep showing it.
export function applyTheme(theme: Theme): void {
  following = theme;
  const query = windowsDark();
  document.documentElement.dataset.theme = shadeOf(theme, query.matches);
  if (!listening) {
    listening = true;
    query.addEventListener("change", (event) => {
      document.documentElement.dataset.theme = shadeOf(following, event.matches);
    });
  }
}
