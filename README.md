# Bot Ross

Evolutionary painting program. It reproduces a reference image with brush strokes, one stroke at a time: for each stroke it tries random candidates, mutates the best, and keeps it only if the painting gets closer to the reference. Strokes are rendered with [libmypaint](https://github.com/mypaint/libmypaint) plus a bristle and paint-load model on top, so the marks look like real brushes.

## Gallery

| Reference | Painting | Preset |
|---|---|---|
| ![reference 1](demo/ref1.jpg) | ![landscape](demo/demo1.jpg) | [`demo1.toml`](demo/demo1.toml) |
| ![reference 2](demo/ref2.jpg) | ![landscape](demo/demo2.jpg) | [`demo2.toml`](demo/demo2.toml) |
| ![reference 3](demo/ref3.jpg) | ![pencil sphere](demo/demo3.jpg) | [`demo3.toml`](demo/demo3.toml) |
| ![reference 4](demo/ref4.jpg) | ![painterly portrait](demo/demo4.jpg) | [`demo4.toml`](demo/demo4.toml) |
| ![reference 5](demo/ref5.jpg) | ![etching](demo/demo5.jpg) | [`demo5.toml`](demo/demo5.toml) |
| ![reference 6](demo/ref6.jpg) | ![graded watercolour](demo/demo6.jpg) | [`demo6.toml`](demo/demo6.toml) |
| ![reference 7](demo/ref7.jpg) | ![realistic watercolour](demo/demo7.jpg) | [`demo7.toml`](demo/demo7.toml) |
| ![reference 8](demo/ref8.jpg) | ![oil and knife portrait](demo/demo8.jpg) | [`demo8.toml`](demo/demo8.toml) |
| ![reference 9](demo/ref9.jpg) | ![sunflower on soft focus](demo/demo9.jpg) | [`demo9.toml`](demo/demo9.toml) |

## Quickstart

Requires Rust and the libmypaint brush engine with its standard brush set, both found through `pkg-config` at build time:

```sh
brew install libmypaint mypaint-brushes
brew install ffmpeg    # only for --video
make build
./target/release/bot-ross demo/ref8.jpg --size 1024 --config demo/demo8.toml --out dog.png
```

A window shows the painting as it grows and `dog.png` is written when it finishes; leave out `--out` to only watch. Swap in any `demo/refN.jpg` with its `demo/demoN.toml` to paint another gallery entry; always paint the reference, not the finished `demoN.jpg`. `make demo` renders every gallery painting missing from `demo/`. `make video` records a time-lapse `demo/demoN.mp4` for each one missing, a frame every `VIDEO_STEP` strokes (default 10).

## Running

```sh
./target/release/bot-ross photo.jpg --size 1024 --config examples/oil_portrait.toml --out painting.png
```

| Flag | Meaning |
|---|---|
| `<reference>` | image to paint, JPEG or PNG |
| `--size` | longest side of the painting in pixels; every size in the config is in these pixels |
| `--config` | painting configuration, see below |
| `--out` | optional image written when painting ends or the window is closed, PNG or JPEG by extension; nothing is saved without it |
| `--seed` | overrides the config seed; same config and seed give the same painting |
| `--layers` | comma-separated layer names to paint, in config order; others are left out |
| `--headless` | paint without the live window |
| `--video` | optional video of the painting growing, encoded by `ffmpeg`, which must be on the path; the finished painting is held for two seconds at the end |
| `--video-step` | strokes per video frame at 30 fps; default 1 |

A window shows the painting as it grows. Close it or press Escape to stop early; with `--out` the painting so far is still saved.

```sh
make run                 # paints examples/reference.png with examples/quick.toml
make run CONFIG=examples/oil_portrait.toml SIZE=1024
make demo                # paints every demo/demoN.toml over demo/refN.jpg into demo/demoN.jpg
make strokes             # one stroke per brush preset, stacked
make bench               # 1,000 random strokes per brush preset with timings
make test
make sketch BRUSH=brushes/watercolour_wet.toml   # draw by hand with a brush preset
make mcp                 # canvas for agents: an MCP server that draws with any brush on request
```

## Sketching with a brush

`sketch` opens a blank canvas where you draw strokes by hand with any brush preset, to see what a brush does before putting it in a config. Each mouse drag becomes one stroke, fitted to a curve through the drag; a click makes a dab.

```sh
cargo run --release --bin sketch -- --brush brushes/watercolour.toml --size 1000 --out sketch.png --blend multiply --radius 20 --opacity 0.6 --colour "#1f3a5f" --background "#ffffff"
```

Every flag but `--brush` is optional with the default shown. While drawing, `[` and `]` change the radius, `-` and `=` the opacity, `C` cycles through a small palette, `P` cycles the pressure shape, `X` clears the canvas, `S` saves, and Escape saves and quits. The current settings are printed whenever they change.

## Canvas for agents

`make mcp` starts an [MCP](https://modelcontextprotocol.io) server over stdio that gives a language model a blank canvas and the brushes. The window shows the canvas as the agent draws; add `--headless` to the command in the Makefile to run without it. Register it with your client, for example `claude mcp add bot-ross -- make -C /path/to/bot-ross mcp`.

| Tool | What it does |
|---|---|
| `new_canvas` | blank canvas of a given `width`, `height` and `background`, with one normal layer |
| `list_brushes` | the presets in `brushes/` by name |
| `draw` | paints `strokes` on the top layer with a `brush`; each stroke has `points`, `radius`, `colour`, `opacity` and `pressure`, and is one smooth curve fitted through its points, rendered with the brush's full physics |
| `add_layer` | a new top layer with `normal` or `multiply` blend |
| `view` | the canvas as a PNG the model can look at, optionally cropped to a `region` and scaled to `max_side` |
| `save` | writes the canvas to a PNG `path` |
| `clear` | wipes the canvas back to its background |

The server is the `bot-ross-mcp` crate in `mcp/`, a workspace member built on the library; `make test` covers it.

## Configuration

A painting is a TOML file. Brushes, stroke types, shape types and regions are defined once by name and assigned to layers, or written inline on a layer. A layer paints either strokes through a brush or polygons with a fill. All sizes are pixels at the working resolution. Ranges are always `[min, max]`; everything else is a single value.

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
radius = [10, 30]                # half-width in px, required; a mark paints about 2 × radius + 1 px wide, and never thinner than at 0.6
length = [60, 300]               # required; or "full" for a straight line across the whole canvas through its point, ending one radius past each edge
length_bias = 0.5                # skews the pick within that range: 0.5 is even, toward 0 favours short strokes, toward 1 long ones; default 0.5
bend = [0.0, 0.3]                # control-point offset as a fraction of length; default [0.0, 0.3]
pressure = "hand"                # "hand" | "flat" | "fading"; default "hand"
jitter = 0.05                    # colour mutation step; default 0.05
contrast = 1.0                   # stroke lightness spread around mid grey: 0 flattens every stroke to mid, 2 pushes light strokes to white and dark ones to black; default 1.0
saturation = 1.0                 # palette: multiplies the sampled colour's saturation, 0 paints in grey; default 1.0
value = 1.0                      # palette: multiplies the sampled colour's lightness, 0 paints every stroke black like ink; default 1.0
palette = ["#8fa3b8", "#c8a85a"] # optional #rrggbb colours; each stroke takes one at random instead of sampling the reference, then the keys above still apply
follow = 0.6                     # 0..1 attraction of stroke direction to contours in the reference; default 0
opacity = 1.0                    # multiplier on the brush's opacity, for glazing; default 1.0
aim = [512, 400]                 # optional point in px; with follow, strokes run toward or away from it instead of along contours
angle = 45                       # optional heading in degrees, single value or [min, max]: 0 points right, 90 down; replaces the random or contour heading, and follow still bends the stroke along its length

[shapes.tile]
points = [4, 6]                  # vertex count, single integer or [min, max]; at least 3
radius = [12, 40]                # circumradius in px, required
irregular = 0.25                 # 0..1 how far vertices stray from even spacing and from the full radius; 0 gives regular polygons, 1 any lopsided shape; default 0
follow = 0.8                     # 0..1 how strongly the shape grabs the patch it sits on: vertices stop where the colour changes and the shape turns along the contour; default 0
aspect = [0.1, 1.0]              # short axis over long axis, single value or range picked per shape; 0.1 gives thin flakes along the rotation; default 1
overlap = false                  # false keeps every shape off the space earlier shapes of the same layer already cover, so they tile; default true
soft = [10, 40]                  # edge width in px over which the shape fades into its surroundings, single value or range; default 0 for crisp edges
jitter = 0.03                    # colour mutation step; default 0.05
contrast = 1.0                   # as for strokes; default 1.0
saturation = 1.0                 # as for strokes; default 1.0
value = 1.0                      # as for strokes; default 1.0
palette = []                     # as for strokes
opacity = 1.0                    # default 1.0
fill = { style = "hatch", spacing = [3, 6], width = 1.2, angle = [30, 60], cross = true }   # default solid, see Shapes

[regions.face]
weight = 3.0                     # error multiplier where the region selects; default 3.0
outside = 1.0                    # multiplier where it does not; below 1 lets the edges stay rough or unfilled; default 1.0
feather = 0.5                    # soft edge as a fraction of each selection's own size: the radius of an ellipse, half the shorter side of a rectangle or polygon, the width of a range; default 0.5
relative = false                 # true reads points as fractions of the canvas width and height and radii as fractions of its shorter side, for configs that must fit any framing; default false
centre = [512, 380]              # shorthand for one ellipse selection
radius = 260
aspect = 1.0                     # height of that ellipse over its width; default 1
select = [                       # selections, combined in order like Lightroom masks; keys within one selection intersect
  { ellipse = { centre = [512, 380], radius = 260, aspect = 1.2 } },
  { rectangle = { from = [0, 700], to = [1024, 1024] }, feather = 2.0 },   # feather overrides the region's for this selection
  { polygon = [[100, 900], [500, 700], [900, 900]] },
  { luminance = [0.0, 0.35] },                                # dark pixels of the reference, 0 to 1
  { hue = [0.55, 0.7], saturation = [0.3, 1.0] },             # blues; hue runs 0 to 1 around the circle and wraps when from is above to
  { mode = "subtract", ellipse = { centre = [512, 300], radius = 80 } },   # "add" | "subtract" | "intersect"; default "add"
]

[[layers]]
name = "blocking"
brush = "oil"                    # name, or inline { file = "..." }
stroke = "bold"                  # name, or inline { radius = [..], length = [..] }
count = 400                      # marks to place; with fill, omitted or 0 paints the whole lattice and a value caps it
region = "face"                  # optional
clip = false                     # true keeps the layer's paint inside its region, in the search and on the canvas; needs a region; default false
placement = "error"              # "error" | "edges" | "random" | "fill"; default "error"
spacing = 8                      # fill only: distance between rows of marks in px
scatter = 0.25                   # fill only: random offset of each mark as a fraction of the spacing; default 0.25
blend = "normal"                 # "normal" | "multiply": how the layer composites over the painting below, in the search and in the result; default "normal"
reference = { blur = 12, smooth = 6, edges = 2.0, levels = 6, contrast = 1.2, saturation = 1.3, gamma = 1.0 }   # optional target tweaks, see below
accident = 0.0                   # chance to keep a stroke that does not lower the error, for deliberate marks; default 0
skip = false                     # true leaves the layer out of the painting; default false
candidates = 8                   # optional per-layer override
refinements = 1                  # optional per-layer override
stroke_radius = [5, 12]          # any stroke key prefixed with stroke_ overrides that key of the layer's stroke
region_weight = 5.0              # likewise region_ for the layer's region

[[layers]]
name = "tiles"
shape = "tile"                   # name, or inline { points = .., radius = [..] }; replaces brush and stroke
count = 1200
shape_fill = { style = "dots", spacing = 4, radius = 1.2 }   # shape_ overrides work the same way
```

Each layer is painted on its own transparent layer over everything below, so a painting is built from coarse to fine. A layer's `blend` chooses how it meets the painting below: `normal` covers it like opaque paint, while `multiply` passes light through it like transparent pigment, so a mark can only darken what is already there and layered colours mix as glazes; the marks within a multiply layer combine the same way, and the search scores candidates with the layer's blend. A layer's `reference` table changes what it is judged against: `blur` softens the target so broad strokes are scored on masses rather than detail they cannot paint, and stroke colours are sampled from the softened target too; `edges` multiplies the error along contours by one plus that factor so detail layers concentrate on getting edges right. `smooth` runs an edge-preserving filter with that spatial sigma in pixels, which flattens shading and noise inside a surface while keeping real boundaries sharp, the basis of a clean stylised look; `levels` posterises the target into that many tones per channel, `contrast` and `saturation` scale it around mid grey, for flat, clearly separated planes in a stylised look, and `gamma` above 1 lifts the midtones so a dark subject can be drawn lighter than life, as an etcher would, while below 1 deepens them; blur is applied first, so a small blur before posterising gives clean regions with crisp boundaries. All default to leaving the reference untouched. Stroke colour is sampled from the reference under the stroke, or drawn from the stroke's `palette` when one is given, then scaled by the stroke's `saturation`, `value` and `contrast`, and mutated by `jitter` in value and saturation, with a small hue drift. A layer's `reference = { saturation = 0 }` is different: it makes the layer judge the painting against a grey target, so the whole result turns monochrome, while a stroke's own `saturation` only changes the palette of its marks. `follow` makes a stroke trace the reference's contours: its direction starts near the local contour tangent and is pulled back toward it at every step along its length, so strokes curve around shapes, with the pull scaled by the value and the local edge strength; flat areas stay random. With `aim` set, the contour field is replaced by straight lines through that point, so strokes lead the eye toward a focus. `placement` decides where strokes go: where the painting is still worst, along edges in the reference, or anywhere; the picked point lands at a random place along the stroke, which starts at most one radius outside the canvas, so edges are painted as densely as the middle. `fill` does not search at all: it lays one mark at every point of a lattice across the region, rows `spacing` apart and, when the stroke has an `angle`, a stroke length apart along the row so lines meet end to end, or one line per row for `length = "full"`, each point kept with probability equal to the region's coverage so feathered edges thin out, and paints every mark without the error test. A pen stroke at 45 degrees over a luminance range hatches the shadows; a wash brush with long level strokes rules a background. A region turns into a weight map once per layer: every selection covers some pixels fully, fades out over its feather, and the selections combine in order, `add` taking the larger coverage, `subtract` removing what the selection covers and `intersect` keeping only what both cover, so a face oval minus the mouth, the dark side of a figure, or the sky by its hue are each one region. The weight is `weight` where the coverage is full and `outside` where it is empty. Weights only steer the search, so strokes that start inside a region can still spill out of it; with `clip = true` every mark is masked by the region's coverage, both when candidates are scored and when they are painted, so an opaque subject can be painted over a watercolour ground without touching it.

Unknown keys and names are errors, and all problems in a file are reported at once. See `examples/quick.toml` for the smallest useful painting, `examples/oil_portrait.toml` for a four-layer oil painting, `examples/digital.toml` for a smooth hyper-realistic digital look, `examples/painterly.toml` and `examples/painterly_portrait.toml` for expressive strokes around a refined face, `examples/stylised.toml` for smooth planes with crisp edges, `examples/focus.toml` for a refined focus surrounded by rigid marks leading toward it, `examples/mosaic.toml` for polygons with every fill style over a knife ground, `examples/pencil.toml` for an etching in ink on paper: drawn contours, hatch families that follow the form and cross-hatched shadows, `examples/pencil2.toml` for the same drawing in white chalk on black paper, which suits dark images with a lit subject, and `examples/watercolour.toml` for a watercolour whose sharpness grades from the bottom of the image to the top, as in a street scene: wet floods and bleeds everywhere, then soft masses, straight vertical and horizontal washes for architecture and crisp patch-grabbing flakes in three bottom-anchored bands with long feathers so detail thins gradually upward, and specks, short dark rigger lines and bright liner accents densest along the bottom, with the wash layers set to `multiply` so the washes glaze one another instead of covering, `examples/hatching.toml` for ink hatching laid by `fill` over a watercolour ground: ruled washes across the sky, hatching on the shadows and cross-hatching on the deepest ones, each a luminance range of the reference, with contours drawn along edges, and `examples/watercolour_realistic.toml` for a more literal watercolour landscape: the same floods and bleeds, then foliage built from soft masses, dry-brush texture and short dabs rather than contour lines, a lake laid in horizontal washes, a few dark rigger lines for trunks and branches, and a final sharpening pass of crisp flakes and short bright lines along the edges in a focus band. Configs with a `face` region in pixels expect the face near the middle of a 1024px portrait; move the region for other framings, or use relative regions as the watercolour example does.

## How a stroke is chosen

Candidates are scored with a fast flat ribbon of the stroke's colour and width, not the full brush. Only the winner is painted for real, and it is kept only if the real render lowers the error, otherwise the layer is rolled back. Brushes whose marks differ a lot from a flat ribbon, such as hairs with strong `shade` or scattered knife patches, get more of these rejections; the layer summary prints the count. A brush preset can steer the ribbon with an optional `[proxy]` table (`opacity`, `width`).

## Shapes

A shape layer places polygons instead of strokes. Each candidate is a polygon with `points` vertices around a circle of a random `radius` from the range, at a random rotation. `radius` bounds the whole shape, not its sides: with `irregular` at zero the vertices sit evenly on that circle, and as it rises each vertex drifts toward its own random angle and pulls in toward the centre by its own random amount, so shapes become lopsided and spiky. `aspect` squashes the whole shape across its rotation, so low values give thin slivers and flakes that lie along the contour when the shape follows one. With `follow` above zero a shape grabs the patch it sits on: each vertex reaches out from the centre only until the target's colour departs from the colour under the centre, so the polygon fills the surface of one tone and stops at the next, whether the boundary is a hard edge or a soft gradient, and its rotation, which hatching follows by default, turns along the local contour; both effects scale with the value, so a shape at 0.5 is halfway between its random outline and the patch. Colour is sampled from the reference around the centre, then `saturation`, `value`, `contrast` and `jitter` apply as for strokes. The search mutates position, rotation, radius, one vertex, colour and the vertex count within the range. `soft` widens the edge from crisp to a fade over that many pixels, which turns a solid shape into a watercolour wash. With `overlap = false` a shape is clipped to whatever its own layer has not painted yet, in the search and on the canvas alike, so shapes tile around each other like cut paper; other layers are ignored. The polygon is rasterised directly, so the search scores exactly what gets painted and shape layers never reject a stroke after painting.

`fill` chooses how the polygon is painted; every style has its own keys, and each numeric key takes a single value or a `[min, max]` range from which every shape draws its own value.

| style | keys |
|---|---|
| `solid` | none; the default |
| `hatch` | `spacing` between lines in px; `width` of a line, default 1; `angle` in degrees the lines run along, by default the polygon's own rotation; `cross = true` adds the perpendicular set |
| `dots` | `spacing` of the dot grid in px; `radius` of a dot, default 1 |
| `outline` | `width` of the edge line, default 1 |

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

The `[line]` tool draws the stroke as a plain anti-aliased line of the stroke's colour at any radius, including sub-pixel ones that paint fainter rather than thinner; it has one key, `opacity`. It ignores the engine and paint sections and is exact for the search, so it never rejects a stroke.

The `[wash]` tool floods the stroke with transparent pigment the way a loaded watercolour brush does. The mark is a field of pigment density: the water spreads past the stroke into lobes, pigment settles into a darker rim where the wash dries, the paper's tooth granulates the interior, and pooling pulls the colour warm or cool from place to place. Opacity grows with density like layers of pigment, so overlapping washes darken each other instead of covering, and the colour darkens where pigment concentrates. The rim and the crisp edge only form on dry paper: where a wash lands on paint already in its layer it is wet-in-wet, so its edge bleeds softly into the earlier wash and no rim is left between them. The stroke's `opacity` sets the strength of the wash and its `[paint]` section the gradient along it: `depletion` and `fade` let a wash run pale toward the end.

| Key | Meaning |
|---|---|
| `bloom` | how far the water spreads past the stroke, as a fraction of the radius; also how lobed the outline is |
| `softness` | typical width of the outer edge as a fraction of the radius; the width varies along the edge, so small values give the crisp line of a dried wash and large ones an irregular gradient, crisp in places and diffuse in others |
| `rim` | extra pigment settled along the edge; 0 for wet-in-wet washes with no outer line |
| `granulation` | fine paper texture in the interior |
| `pooling` | slow density variation across the wash |
| `drift` | hue shift, as a fraction of the hue circle, between pooled and thin areas |

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
- **Pencil** is a single hard hair whose width and opacity follow pressure, so strokes taper at both ends, with paper grain breaking the graphite and a little unevenness along the line; `examples/example5.toml` hatches a sphere with it, one direction per plane and feathered luminance bands from light to dark.
- **Airbrush** is a single very soft, low-opacity hair for smooth gradients, and **Soft round** a semi-soft one for detail without hard patches; `examples/digital.toml` builds a hyper-realistic digital look from them and the pen.
- **Palette knife** is a rigid blade: an irregular, sharp-edged vector tip swept along the stroke as a single flat sheet, with scrape streaks, pushed paint, and paint landing in scattered patches that open up as the blade runs dry.
- **Watercolour** is a wash dried on the paper: pigment blooms past the stroke in lobes, dries to a darker rim, granulates on the paper and pools warm or cool, and runs pale as the brush empties.
- **Wet watercolour** is the same wash laid wet-in-wet: no rim, a wide and irregular gradient at the edge and stronger pooling, for skies and soft masses; `examples/watercolour.toml` floods the background and glazes the focus with it and keeps the dried wash for masses, accents and drops.

## Features

- Greedy per-stroke search with mutation, exact incremental error over the stroke's bounds
- Layers assigned a brush and a stroke type, or a polygon shape type, with named reusable definitions and per-layer key overrides
- Polygons with a vertex count range, radius range and irregularity, filled solid, hatched, dotted or outlined
- Strokes follow cubic Bézier curves with a pressure profile that eases in and out like a hand
- Strokes rendered by libmypaint with a bristle and paint-load model on top
- Brushes defined in TOML: engine settings, input dynamics, bristles, a vector stamp or a watercolour wash, paint
- Wet oil, dry brush, palette knife, dried and wet watercolour, calligraphy, pen, pencil, airbrush, soft round, hard round and liner presets
- Live window while painting, PNG output, optional snapshots and a time-lapse video through ffmpeg
- MCP server giving agents a canvas to draw on with any brush and to look at
