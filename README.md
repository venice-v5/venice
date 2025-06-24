# Venice

Open source MicroPython port for VEX V5 robots.

## Note

Builds are still a little bit weird. If you get any compilation errors after modifying the port
files, try running `cargo clean` and recompiling.

## Roadmap

- [x] V5 binary with MicroPython embedded running a static program
- [ ] CLI tool
- [x] Bytecode loading at runtime through VEXos linking
- [ ] Multi-python-module support
- [ ] Python V5 API
- [ ] Multitasking (async/await?)
