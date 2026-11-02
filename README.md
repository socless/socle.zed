# socle.zed

A [Zed](https://zed.dev/) extension for [Socle](https://github.com/socless/socle),
the Chisel IDE language server. In a Chisel project built with Mill, it adds a
**View design** and a **Simulate** code lens above every ChiselSim test:

- **View design** elaborates that test's design and shows it in a live panel in
  your browser. Saving a `.scala` file refreshes the panel, and a click on a box
  in the panel jumps to its source.
- **Simulate** runs the test with waveforms on and shows the trace in Surfer,
  following the panel's selection.

The extension downloads the Socle server release it is tested against,
checks its sha256, and starts it next to Metals. Your tests and build are never
modified. The server itself is described in Socle's
[protocol page](https://github.com/socless/socle/blob/main/docs/lsp-protocol.md).

Compared with VS Code and Neovim, the panel does not follow the cursor: that
needs a custom LSP notification, which Zed extensions cannot send.

## Private repository

Both repositories (socless/socle and socless/socle.zed) are private for now, so
the extension is not in Zed's registry, and the server jar download fails (404).
Until they are public:

1. Build the server jar in a Socle checkout and copy it out of `out/`:

   ```bash
   ./mill socle-lsp.assembly
   cp out/socle-lsp/assembly.dest/out.jar ~/.local/share/socle/socle-lsp.jar
   ```

2. Point the extension at it in Zed's settings (see [Settings](#settings)):

   ```jsonc
   "lsp": { "socle": { "binary": { "arguments": ["-jar", "/home/you/.local/share/socle/socle-lsp.jar"] } } }
   ```

Without it, the server does not start and Zed's log
(`zed: open log`) says `Socle: cannot download ...` with this hint.

## Requirements

- Zed with LSP code lens support (May 2026 or later; tested with 1.25.3).
- The [Scala extension](https://zed.dev/extensions/scala), for the Scala language.
- Java 21 or later: `lsp.socle.binary.path`, else `$JAVA_HOME/bin/java`, else
  `java` on `PATH`.
- A Mill project opened at its root: a `build.mill` there and `./mill`. In any
  other worktree the server does not start (`no build.mill at the root of this
  worktree` in Zed's log).
- For Simulate only: Verilator on `PATH`, and the
  [Surfer](https://surfer-project.org/) app unless `waveViewer` is `"surfer-panel"`.
- Linux and macOS. Windows is not tested.

## Install

While the repository is private, install it as a dev extension. Zed builds it
with Rust, so you need [rustup](https://rustup.rs/) (the
`rust-toolchain.toml` here picks the toolchain and its `wasm32-wasip2` target):

1. `git clone git@github.com:socless/socle.zed.git`
2. In Zed: `zed: extensions`, then **Install Dev Extension**, and pick the clone.

Then turn the code lenses on, which Zed leaves off by default:

```jsonc
"code_lens": "on"
```

`"menu"` puts them in the code actions menu (`editor: toggle code actions`,
`ctrl-.`) instead of above the tests.

Zed starts language servers only in a trusted worktree: accept the prompt when
you open the project.

## Use it

1. Open a ChiselSim test, for example `example/test/src/gcd/GcdSpec.scala` in
   Socle. A `View design` and a `Simulate` lens appear on each test.
2. Click `View design`. The first view opens the panel at
   `http://127.0.0.1:<port>/` in your browser. The first run of a test module
   also compiles it with Mill, so it can take a while; later views take seconds.
3. Edit and save: the last viewed test runs again and the panel refreshes.
4. Click a box in the panel: Zed opens its source at that line.

The server's messages (failed builds, simulation results) show as Zed
notifications; its log is in `dev: open language server logs`, server `socle`.

## Settings

All of them go under `lsp.socle` in Zed's `settings.json`:

```jsonc
"lsp": {
  "socle": {
    "binary": {
      "path": "/usr/lib/jvm/java-21-openjdk/bin/java",           // the java to run
      "arguments": ["-jar", "/path/to/socle-lsp.jar"],        // a local jar, instead of the download
      "env": { "PATH": "/opt/verilator/bin:/usr/bin:/bin" }     // the server's environment
    },
    "initialization_options": { "waveFlag": "emitFst", "waveViewer": "surfer-panel" }
  }
}
```

- `binary.arguments`, when set, are all the arguments `java` gets: the pinned
  jar is then not downloaded. Without them, the server runs as
  `java -Djava.net.preferIPv4Stack=true -jar <pinned jar>`.
- `binary.path` only picks the `java` executable.
- `initialization_options` are the server's Simulate and waveform settings,
  with the names of Socle's
  [settings table](https://github.com/socless/socle/blob/main/docs/ide-vscode.md#settings)
  without the `socle.` prefix: `waveFlag`, `surferCommand`, `waveViewer`,
  `waveViewerCommand`, `wavesFollowSelection`, `wavesShowClockReset`,
  `wavesFoldBundles`, `wavesShowWires`, `wavesColorByDirection`,
  `wavesFollowDelayMs`. A change applies without restarting the server.

## The pinned server

The extension downloads Socle 0.22.0's `socle-lsp-0.22.0.jar` once, into
its own directory, and runs it after checking its sha256. Both are in
`src/server.rs` (`PINNED` and `PINNED_SHA256`); to bump the server, change both
and this README, and bump the extension's version in `extension.toml` and
`Cargo.toml`.

## Development

```bash
cargo test                                    # the unit tests (host)
cargo build --release --target wasm32-wasip2  # the extension, as Zed builds it
```

`src/server.rs` holds the logic that does not need Zed (the pin, the java and
arguments, the messages) and its tests; `src/lib.rs` is the Zed side (the
download, the settings). After a change, **Rebuild** on the extension's card
in `zed: extensions` picks it up.

## License

MIT
