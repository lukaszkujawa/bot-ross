# Bot Ross

Evolutionary painting program. It reproduces a reference image with brush strokes, one stroke at a time: for each stroke it tries random candidates, mutates the best, and keeps it only if the painting gets closer to the reference. Strokes are rendered with [libmypaint](https://github.com/mypaint/libmypaint) plus a bristle and paint-load model on top, so the marks look like real brushes.

## Installation

Requires Rust and the libmypaint brush engine with its standard brush set:

```sh
brew install libmypaint mypaint-brushes
make build
```

Both libraries are located through `pkg-config` at build time.

## Running

```sh
bot-ross photo.jpg --size 1024 --config examples/oil_portrait.toml --out painting.png
```

| Flag | Meaning |
|---|---|
| `<reference>` | image to paint, JPEG or PNG |
| `--size` | longest side of the painting in pixels; every size in the config is in these pixels |
| `--config` | painting configuration, see below |
| `--out` | PNG written when painting ends or the window is closed |
| `--seed` | overrides the config seed; same config and seed give the same painting |
| `--headless` | paint without the live window |

A window shows the painting as it grows. Close it or press Escape to stop early; the painting so far is still saved.

```sh
make run                 # paints examples/reference.png with examples/quick.toml
make run CONFIG=examples/oil_portrait.toml SIZE=1024
make demo                # one stroke per brush preset, stacked
make bench               # 1,000 random strokes per brush preset with timings
make test
```

## Configuration

A painting is a TOML file. Brushes, stroke types and regions are defined once by name and assigned to layers, or written inline on a layer. All sizes are pixels at the working resolution. Ranges are always `[min, max]`; everything else is a single value.

```toml
[canvas]
background = "mean"              # "mean" | "white" | "#rrggbb"; default "mean"

[search]
candidates = 16                  # random strokes tried per placed stroke; default 16
refinements = 3                  # mutation rounds on the best candidate; default 3
patience = 4                     # stop a layer after count * patience failed attempts; default 4
seed = 0                         # default 0

[output]
report = 100                     # progress line every N placed strokes; default 100
snapshots = "frames"             # optional directory, PNG at every report

[brushes.oil]
file = "../brushes/wet_oil.toml" # brush preset, relative to this file

[strokes.bold]
radius = [10, 30]                # required
length = [60, 300]               # required
bend = [0.0, 0.3]                # control-point offset as a fraction of length; default [0.0, 0.3]
pressure = "hand"                # "hand" | "flat" | "fading"; default "hand"
jitter = 0.05                    # colour mutation step; default 0.05

[regions.face]
centre = [512, 380]
radius = 260
weight = 3.0                     # error multiplier inside the circle; default 3.0
outside = 1.0                    # multiplier far outside; below 1 lets the edges stay rough or unfilled; default 1.0
feather = 0.5                    # fraction of the radius over which weight blends into outside; default 0.5

[[layers]]
name = "blocking"
brush = "oil"                    # name, or inline { file = "..." }
stroke = "bold"                  # name, or inline { radius = [..], length = [..] }
count = 400
region = "face"                  # optional
placement = "error"              # "error" | "edges" | "random"; default "error"
candidates = 8                   # optional per-layer override
refinements = 1                  # optional per-layer override
```

Each layer is painted on its own transparent layer over everything below, so a painting is built from coarse to fine. Stroke colour is sampled from the reference under the stroke and mutated by `jitter`. `placement` decides where strokes start: where the painting is still worst, along edges in the reference, or anywhere.

Unknown keys and names are errors, and all problems in a file are reported at once. See `examples/quick.toml` for the smallest useful painting, `examples/oil_portrait.toml` for a four-layer oil painting, `examples/digital.toml` for a smooth hyper-realistic digital look, and `examples/painterly.toml` for expressive strokes around a refined face. Configs with a `face` region expect the face near the middle of a 1024px portrait; move the region for other framings.

## How a stroke is chosen

Candidates are scored with a fast flat ribbon of the stroke's colour and width, not the full brush. Only the winner is painted for real, and it is kept only if the real render lowers the error, otherwise the layer is rolled back. Brushes whose marks differ a lot from a flat ribbon, such as hairs with strong `shade` or scattered knife patches, get more of these rejections; the layer summary prints the count. A brush preset can steer the ribbon with an optional `[proxy]` table (`opacity`, `width`).

## Brushes

A brush is a TOML file with three layers of control. The engine layer is libmypaint: `template` names an installed MyPaint brush to start from, `settings` overrides any of its base values by name and `dynamics` maps any input (pressure, speed1, random, stroke, direction, ...) onto any setting as a curve. An empty curve removes a template's mapping.

```toml
name = "Palette knife"
template = "classic/wet_knife"

[settings]
hardness = 1.0
elliptical_dab_ratio = 4.0

[dynamics]
elliptical_dab_angle = { direction = [[0.0, 0.0], [180.0, 180.0]] }
radius_logarithmic = { pressure = [] }
```

A brush paints with one of two tools. The `[bristles]` tool splits every stroke into individual hairs, each traced as its own thin line through libmypaint. This is what the engine cannot do on its own and what makes hair marks visible.

The `[stamp]` tool sweeps a vector brush tip along the stroke instead, for rigid tools like a knife. Tips are SVG files in `brush_tips/`; the tip's x axis runs along the stroke and its height spans the stroke width. The whole sweep is composited as one mark, so edges stay as sharp as the tip's outline.

| Key | Meaning |
|---|---|
| `tip` | path to the SVG tip |
| `spacing` | distance between stamps as a fraction of the stroke radius |
| `ripple` | slow variation of the blade width along the stroke |
| `wobble` | how far the blade wanders sideways over the stroke |
| `streaks` | strength of scrape streaks running along the stroke |
| `smear` | how much paint already on the canvas is pushed into the mark |

The paint layer models how much paint the tool carries and how it leaves the canvas.

| Key | Meaning |
|---|---|
| `load` | paint on the brush at the start |
| `depletion` | how much is used up by the end of the stroke |
| `fade` | fraction of the stroke over which opacity eases to zero at the end; smaller fades faster |
| `unevenness` | slow random variation of deposited paint along a hair |
| `skip`, `grain` | fraction of the mark that misses the canvas, and how fine the gaps are; gaps grow as paint runs out. For bristles a hair skips in runs along its length; for a stamp paint lands in scattered patches with sharp torn edges |

Presets in `brushes/` follow the physics of each tool:

- **Wet oil** is fully loaded, so hairs cover the body and furrows come from lightness differences between overlapping hairs; moderate smudge lets later hairs drag earlier paint.
- **Dry brush** carries little paint, so many very thin hairs deposit only on canvas high points, break into fragments and fade as they run dry, with no smudging.
- **Calligraphy** is a single flat nib held at a fixed angle, so the line is thick across the nib and thin along it, and swells with pressure.
- **Pen** is a single monoline: constant width and full opacity whatever the pressure.
- **Airbrush** is a single very soft, low-opacity hair for smooth gradients, and **Soft round** a semi-soft one for detail without hard patches; `examples/digital.toml` builds a hyper-realistic digital look from them and the pen.
- **Palette knife** is a rigid blade: an irregular, sharp-edged vector tip swept along the stroke as a single flat sheet, with scrape streaks, pushed paint, and paint landing in scattered patches that open up as the blade runs dry.

## Features

- Greedy per-stroke search with mutation, exact incremental error over the stroke's bounds
- Layers assigned a brush and a stroke type, with named reusable definitions
- Strokes follow cubic Bézier curves with a pressure profile that eases in and out like a hand
- Strokes rendered by libmypaint with a bristle and paint-load model on top
- Brushes defined in TOML: engine settings, input dynamics, bristles or a vector stamp, paint
- Wet oil, dry brush, palette knife, calligraphy, pen, airbrush and soft round presets
- Live window while painting, PNG output, optional snapshots
