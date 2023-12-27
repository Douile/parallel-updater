<head>
<link rel="stylesheet" href="https://doc.rust-lang.org/static.files/rustdoc-fa3bb1812debf86c.css">
</head>

# Parallel updater (WIP)

This program allows you to define commands to run an update (e.g. `pacman -Syu`) and then run them in parallel. The added benefit of this program over a shell script is that it has knowledge of the requirements of each update:

- Whether input is needed (e.g. to confirm updates or enter sudo password)
- Whether root is needed (through sudo or other means)
- Dependencies and conflicts
  - Which updates need another update to run first
  - Which cannot run at the same time as others (e.g. AUR update/repo update)

Using this knowledge it greedily chooses which to run next aiming for maximum parallelism (things that need input cannot run at same time, but when an update is done needing input another update that needs input can start).

## Features

- [x] Parallel CLI updates
- [x] Greedy update selection based on STDIN usage
- [x] Config file parsing
- [x] Input needed detection (Based on stdout)
- [ ] Use the update's ID for nicer logging
- [ ] STDIN read detection (buffer empty / unbuffered pipe)
- [ ] Sudo keep-alive (see paru)
- [ ] Call programs via dynamic libraries rather than CLI (e.g. libparu)
- [ ] Save update logs to files
- [ ] Notify user when input is needed
- [ ] Notify user when updates are done / run a specific command when updates are done
- [ ] Priority configuration (which to run first)
- [ ] Better live status of what is currently being updated (when stdin not in use)
  - [ ] When stdin not in use allow cancelling of updates via user input
- [ ] CLI update filtering
  - [ ] Group configuration (allow running updates in specific groups)
  - [ ] Ignore specific updates

## Configuring

When you run the program it looks for a file called `updates.toml` in the current directory, or in `~/.config/parallel-update-cli/updates.toml` (This can be overriden with the `--config-file [file]` option).

The config file format is yet to be finalized but currently it is:

# Program configuration

```toml
[updater]
# ...
```

<h3 id="fields" class="fields small-section-header">
            Fields<a href="#fields" class="anchor">§</a>
          </h3>
          <span
            id="structfield.output_duration"
            class="structfield small-section-header"
            ><a href="#structfield.output_duration" class="anchor field">§</a
            ><code
              >output_duration:
              <a
                class="primitive"
                href="https://doc.rust-lang.org/1.74.1/std/primitive.bool.html"
                >bool</a
              ></code
            ></span
          >
          <div class="docblock"><p>Output how long each update took</p></div>
          <span
            id="structfield.output_success_logs"
            class="structfield small-section-header"
            ><a href="#structfield.output_success_logs" class="anchor field"
              >§</a
            ><code
              >output_success_logs:
              <a
                class="primitive"
                href="https://doc.rust-lang.org/1.74.1/std/primitive.bool.html"
                >bool</a
              ></code
            ></span
          >
          <div class="docblock">
            <p>Output stdout/stderr for successful updates</p>
          </div>
          <span
            id="structfield.output_failure_logs"
            class="structfield small-section-header"
            ><a href="#structfield.output_failure_logs" class="anchor field"
              >§</a
            ><code
              >output_failure_logs:
              <a
                class="primitive"
                href="https://doc.rust-lang.org/1.74.1/std/primitive.bool.html"
                >bool</a
              ></code
            ></span
          >
          <div class="docblock">
            <p>Output stdout/stderr for failed updates</p>
          </div>
          <span
            id="structfield.output_states"
            class="structfield small-section-header"
            ><a href="#structfield.output_states" class="anchor field">§</a
            ><code
              >output_states:
              <a
                class="primitive"
                href="https://doc.rust-lang.org/1.74.1/std/primitive.bool.html"
                >bool</a
              ></code
            ></span
          >
          <div class="docblock"><p>Output update states</p></div>
          <span
            id="structfield.threads"
            class="structfield small-section-header"
            ><a href="#structfield.threads" class="anchor field">§</a
            ><code
              >threads:
              <a
                class="primitive"
                href="https://doc.rust-lang.org/1.74.1/std/primitive.usize.html"
                >usize</a
              ></code
            ></span
          >
          <div class="docblock"><p>Number of updates to run at once</p></div>
          <span
            id="structfield.debug_config"
            class="structfield small-section-header"
            ><a href="#structfield.debug_config" class="anchor field">§</a
            ><code
              >debug_config:
              <a
                class="primitive"
                href="https://doc.rust-lang.org/1.74.1/std/primitive.bool.html"
                >bool</a
              ></code
            ></span
          >
          <div class="docblock"><p>Debug config</p></div>

# Invividual update configuration

```toml
[update.id]
# ...
```

<h3 id="fields" class="fields small-section-header">
            Fields<a href="#fields" class="anchor">§</a>
          </h3>
          <span id="structfield.kind" class="structfield small-section-header"
            ><a href="#structfield.kind" class="anchor field">§</a
            ><code
              >kind:
              <a
                class="enum"
                href="#updatekind"
                title="enum parallel_update_config::primatives::UpdateKind"
                >UpdateKind</a
              ></code
            ></span
          >
          <div class="docblock"><p>The kind of the update</p></div>
          <span id="structfield.input" class="structfield small-section-header"
            ><a href="#structfield.input" class="anchor field">§</a
            ><code
              >input:
              <a
                class="primitive"
                href="https://doc.rust-lang.org/1.74.1/std/primitive.bool.html"
                >bool</a
              ></code
            ></span
          >
          <div class="docblock">
            <p>Whether the update requires exclusive input</p>
          </div>
          <span id="structfield.root" class="structfield small-section-header"
            ><a href="#structfield.root" class="anchor field">§</a
            ><code
              >root:
              <a
                class="primitive"
                href="https://doc.rust-lang.org/1.74.1/std/primitive.bool.html"
                >bool</a
              ></code
            ></span
          >
          <div class="docblock">
            <p>Whether the update uses a program that gives root (e.g. sudo)</p>
          </div>
          <span
            id="structfield.conflicts"
            class="structfield small-section-header"
            ><a href="#structfield.conflicts" class="anchor field">§</a
            ><code
              >conflicts:
              <a
                class="struct"
                href="https://doc.rust-lang.org/1.74.1/alloc/vec/struct.Vec.html"
                title="struct alloc::vec::Vec"
                >Vec</a
              >&lt;<a
                class="struct"
                href="https://doc.rust-lang.org/1.74.1/alloc/string/struct.String.html"
                title="struct alloc::string::String"
                >String</a
              >&gt;</code
            ></span
          >
          <div class="docblock">
            <p>
              Updates that cannot run at the same time (order doesn’t matter)
            </p>
          </div>
          <span
            id="structfield.depends"
            class="structfield small-section-header"
            ><a href="#structfield.depends" class="anchor field">§</a
            ><code
              >depends:
              <a
                class="struct"
                href="https://doc.rust-lang.org/1.74.1/alloc/vec/struct.Vec.html"
                title="struct alloc::vec::Vec"
                >Vec</a
              >&lt;<a
                class="struct"
                href="https://doc.rust-lang.org/1.74.1/alloc/string/struct.String.html"
                title="struct alloc::string::String"
                >String</a
              >&gt;</code
            ></span
          >
          <div class="docblock"><p>Updates that must run before</p></div>
                    <span id="structfield.exe" class="structfield small-section-header"
            ><a href="#structfield.exe" class="anchor field">§</a
            ><code
              >exe:
              <a
                class="struct"
                href="https://doc.rust-lang.org/1.74.1/alloc/string/struct.String.html"
                title="struct alloc::string::String"
                >String</a
              ></code
            ></span
          >
          <div class="docblock"><p>Path to executable of program</p></div>
          <span id="structfield.argv" class="structfield small-section-header"
            ><a href="#structfield.argv" class="anchor field">§</a
            ><code
              >argv:
              <a
                class="enum"
                href="https://doc.rust-lang.org/1.74.1/core/option/enum.Option.html"
                title="enum core::option::Option"
                >Option</a
              >&lt;<a
                class="struct"
                href="https://doc.rust-lang.org/1.74.1/alloc/vec/struct.Vec.html"
                title="struct alloc::vec::Vec"
                >Vec</a
              >&lt;<a
                class="struct"
                href="https://doc.rust-lang.org/1.74.1/alloc/string/struct.String.html"
                title="struct alloc::string::String"
                >String</a
              >&gt;&gt;</code
            ></span
          >
          <div class="docblock"><p>Optional arguments for the program</p></div>
          <span
            id="structfield.environ"
            class="structfield small-section-header"
            ><a href="#structfield.environ" class="anchor field">§</a
            ><code
              >environ:
              <a
                class="enum"
                href="https://doc.rust-lang.org/1.74.1/core/option/enum.Option.html"
                title="enum core::option::Option"
                >Option</a
              >&lt;<a
                class="struct"
                href="https://doc.rust-lang.org/1.74.1/alloc/vec/struct.Vec.html"
                title="struct alloc::vec::Vec"
                >Vec</a
              >&lt;(<a
                class="struct"
                href="https://doc.rust-lang.org/1.74.1/alloc/string/struct.String.html"
                title="struct alloc::string::String"
                >String</a
              >,
              <a
                class="struct"
                href="https://doc.rust-lang.org/1.74.1/alloc/string/struct.String.html"
                title="struct alloc::string::String"
                >String</a
              >)&gt;&gt;</code
            ></span
          >
          <div class="docblock">
            <p>Optional extra environment variables for the program</p>
          </div>
          <span
            id="structfield.working_directory"
            class="structfield small-section-header"
            ><a href="#structfield.working_directory" class="anchor field">§</a
            ><code
              >working_directory:
              <a
                class="enum"
                href="https://doc.rust-lang.org/1.74.1/core/option/enum.Option.html"
                title="enum core::option::Option"
                >Option</a
              >&lt;<a
                class="struct"
                href="https://doc.rust-lang.org/1.74.1/alloc/string/struct.String.html"
                title="struct alloc::string::String"
                >String</a
              >&gt;</code
            ></span
          >
          <div class="docblock">
            <p>The directory the program should be executed in</p>
          </div>
          <span
            id="structfield.passthrough_environ"
            class="structfield small-section-header"
            ><a href="#structfield.passthrough_environ" class="anchor field"
              >§</a
            ><code
              >passthrough_environ:
              <a
                class="primitive"
                href="https://doc.rust-lang.org/1.74.1/std/primitive.bool.html"
                >bool</a
              ></code
            ></span
          >
          <div class="docblock">
            <p>Whether to past through the host programs environment.</p>
          </div>

<h3 id="updatekind" class="small-section-header">
    UpdateKind<a href="#updatekind" class="anchor">§</a>
    </h3>
              <pre class="rust item-decl"><code>pub enum UpdateKind {
    Default,
    Paru,
    Input,
}</code></pre>

## Contributing

All contributions are welcome:

- [Bug report/feature request (Issue)](https://github.com/Douile/parallel-updater/issues)
- [Code (Pull Request)](https://github.com/Douile/parallel-updater/pulls)

## License

All code is made available under the [MIT License](./LICENSE), contributions should be made available under a compatible license.
