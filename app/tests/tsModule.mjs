// Runs an `app/src` module under Node's own test runner. Node strips the types of a `.ts` file but
// does not complete an extensionless import the way the bundler does, so a resolver hook adds the
// extension the module meant.
//
// The Tauri bridge is replaced by a fake, so a screen's state and commands run with no window: a
// test fakes the commands it needs and reads back the calls the module made.

import { registerHooks } from "node:module";

registerHooks({
  resolve(specifier, context, next) {
    // `./confirm.svelte` is the rune module `confirm.svelte.ts`.
    if (specifier.startsWith(".") && (!/\.[a-z]+$/.test(specifier) || specifier.endsWith(".svelte"))) {
      for (const extension of [".ts", ".js"]) {
        try {
          return next(specifier + extension, context);
        } catch {
          // Not this extension; try the next one.
        }
      }
    }
    return next(specifier, context);
  },
});

// Replace the bridge: `commands` maps a command name to a function of its arguments. Every call is
// recorded, in order, so a test can assert what the module asked the back end for.
export function useBackend(commands) {
  const calls = [];
  const callbacks = new Map();
  emitters.clear();
  let nextId = 1;
  globalThis.window = {
    // The event plugin's own bookkeeping, which `listen`'s returned unlistener touches.
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
    __TAURI_INTERNALS__: {
      async invoke(command, args) {
        calls.push({ command, args });
        const answer = commands[command];
        if (!answer) throw new Error(`the test faked no ${command} command`);
        if (command === "plugin:event|listen") {
          const handler = callbacks.get(args.handler);
          if (handler) emitters.set(args.event, [...(emitters.get(args.event) ?? []), handler]);
        }
        return answer(args);
      },
      transformCallback(callback) {
        const id = nextId++;
        callbacks.set(id, callback);
        return id;
      },
      unregisterCallback(id) {
        callbacks.delete(id);
      },
    },
  };
  return calls;
}

// The listeners the faked `listen` registered, by event name.
const emitters = new Map();

// Deliver an event the back end would send, so a screen's own handler for it runs.
export function emitEvent(event, payload) {
  for (const callback of emitters.get(event) ?? []) callback({ event, id: 0, payload });
}

// Svelte's runes outside the compiler, so a test can build a component's state and read it back.
// A `$derived` value is taken once, at construction, and does not follow a later change.
export function useRunes() {
  globalThis.$state = (value) => value;
  globalThis.$derived = (value) => value;
}
