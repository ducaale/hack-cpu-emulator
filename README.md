# Hack CPU Emulator

A WIP Re-implementation of Nand2tetris' CPU Emulator in the terminal.

## Requirements
- Rust v1.42 or later

## Usage

Make sure you obtain an assembly file from [nand2Tetris](https://www.nand2tetris.org/software)

```sh
hack-cpu-emulator <assembly file>
```

- Press `n` for one CPU tick.
- Press `j` and `k` for navigating the content of the memory.
- Press `r` to edit the memory at a certain address. Use `Enter` to confirm the edit
  and `Esc` to cancel.
- Press `b` to enter keyboard mode. Use `Esc` to exit the mode.
- Press `f` to toggle maximizing the computer screen.
- Press `q` to quit the program.

## Todos
- [x] Implement screen widget
- [x] Introduce keyboard inputMode
- [ ] Support toggling between view modes (binary, hex, decimal, asm)
- [ ] Implement non-interactive mode
- [ ] Time travel

## Screenshots
![screenshot](https://raw.githubusercontent.com/ducaale/hack-cpu-emulator/master/screenshots/screenshot-1.png)

![screenshot](https://raw.githubusercontent.com/ducaale/hack-cpu-emulator/master/screenshots/screenshot-2.png)