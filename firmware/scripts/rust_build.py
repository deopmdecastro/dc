Import("env")
# Delegates the actual Rust build to `cargo`, mirroring esp-rs/esp-idf-template.
def _run_cargo(source, target, env):
    env.Execute("cargo build --release --locked")

env.AddPreAction("buildprog", _run_cargo)
