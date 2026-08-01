# NEON // LAST STAND

A standalone wave-survival game built with Bevy 0.18.1.

## Run

```powershell
Set-Location C:\Users\moham\Desktop\bevy
cargo run
```

The first run compiles Bevy and may take several minutes. Later runs reuse the
build cache.

## Controls

- `WASD` or arrow keys: move the cyan ship.
- Hold `Space`: fire at the nearest hostile signal.
- `R`: reboot after a game over.

The arena starts with six hostiles. Each cleared wave increases enemy count,
health, and speed. The HUD tracks health, score, kills, and the current wave.
