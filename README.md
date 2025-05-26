# timewatch

A terminal-based countdown timer with adaptive layout.

![screenshot](./assets/output.gif)

# Features

- Displays time in large ASCII digits
- Optional message bellow the target
- Automatically chooses horizontal or vertical layout based on terminal size
- Adapts to terminal resizes
- Can exit early with `q`, `ESC` or `Ctrc+C`
- Can pause the timer using `SPACE`
- Accepts `hh:mm:ss`, `mm:ss` or just `ss` time format

# Installation

```bash
cargo install timewatch
```

# Usage examples

```bash
# Reminder to take a break
timewatch 1:00:00 "Break in:"
# Pomodoro
timewatch 25:00 "Pomodoro Session"
# Combine with sounds (or other commands)
timewatch 10 && play ding.wav
```

Can stop the timewatch anytime using `q`, `ESC` or `Ctrl+C`. Can pause it using
`SPACE`.

# Contributing

PRs and Issues are welcome!

# License

MIT
