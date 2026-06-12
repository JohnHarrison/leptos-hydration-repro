# Minimal repro: tachys hydration panic for a blocking OnceResource created inside a Suspense child

leptos `0.8.19` / tachys `0.2.15`. Reduced from a production page (switchback#500).

```sh
cargo leptos serve   # needs cargo-leptos 0.3.x + wasm32-unknown-unknown
# open http://127.0.0.1:3111, hard reload, watch the console
```

Console output on load:

```
A hydration error occurred while trying to hydrate an element defined at src/lib.rs:78.
The framework expected an HTML <div> element, but found this instead: …
panicked at tachys-0.2.15/src/hydration.rs:184:9: Unrecoverable hydration error.
pageerror: unreachable
```

## The discriminating matrix

`src/lib.rs` is the panicking shape (row 3). Each row was verified by editing
the same file and re-driving a headless Chromium at it:

| # | structure | hydration |
|---|-----------|-----------|
| 1 | `OnceResource::new_blocking` read in a bare `{move \|\| …}` closure, **no Suspense ancestor** — flattened *or* with a nested dynamic block | **PANIC** |
| 2 | resource created **before** a `<Suspense>`, read in a closure inside it — flattened or nested | clean |
| 3 | resource created **inside a `<Suspense>`-rendered dynamic child** (a component returned from the outer suspense closure), read in a bare closure | **PANIC** ← `src/lib.rs` |
| 4 | same as 3, but the inner read wrapped in its own nested `<Suspense>` | clean |

Row 3 is the production shape: an outer "page detail" blocking resource +
`<Suspense>`, whose resolved branch renders a child component that creates its
own blocking resource for a sub-list. SSR streams the resolved markup, but the
client's hydration walk desyncs at the first element produced from the inner
resource's closure.

Notes:
- The value is `new_blocking` and serialized, so both sides should commit the
  same branch at first paint — naively there is nothing to mismatch.
- The panic is unrecoverable; the page (interactivity) is dead afterwards.
- Row 1 also panics, which is harsher than the documented "reading a resource
  outside Suspense" warning suggests.
