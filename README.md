# Microbiome

An Osmos-inspired evolutionary simulation in Rust using macroquad.

## Mechanics

- **Movement**: Cells eject mass to move (conservation of momentum)
- **Eating**: Contact transfers mass from smaller to larger
- **Heat**: Ejecting mass generates heat. Overheat (100%) locks out ejection until cooldown (40%)
- **Mass decay**: Cells shrink over time, forcing engagement
- **Food**: Passive particles spawn to sustain the population
- **Boundaries**: Soft repulsion, no hard walls

## Running

```
cargo run -r
```

## Controls

| Key | Action |
|-----|--------|
| Click | Spectate |
| Ctrl+Click | Control cell |
| Shift+Click | Explode cell |
| Hold+Release | Eject mass (control mode) |
| Right-click/Esc | Release cell from control/spectate |
| Right drag | Pan |
| Scroll | Zoom |
| -/= | Sim speed |
| R | Reset |
| Q | Quit |
