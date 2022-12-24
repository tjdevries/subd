# subd

`subd`'s goal is to create a link between the various ways viewers support you
as a live content creator, and the interactions that can be triggered from
various forms for support.

Interactions:
- Twitch viewers, subs, donators, memers,
- Github Sponsors
- Twitter supporters/shit-posters
- Youtube comment Warriors/Tik-Tok Comment Therapists
- Discord Commentors
- `<more>`

Integrations You Can Trigger:
- OBS
- Browser Source
- Twitch Chat+Moderation
- Discord
- Twitter
- Neovim
- Whatever we want!!!!

## Status

We are developing this on the Twitch streams: [teej_dv](https://twitch.tv/teej_dv) and [beginbot](https://www.twitch.tv/beginbot)

## Development / Running

There are several steps required to set up `subd`. See [Developers](./DEVELOPERS.md) for more information.

----

## Commands

The commands available from Twitch Chat, what they do, and how they work.

### !scroll

```
!scroll SOURCE SCROLL_SETTING speed duration
```

Scroll Settings:

- speed_x
- speed_y

**Real Examples:**
```
// Scroll the "begin source
// along "x" at a speed of "500",
// taking "10000" milliseconds to reach full speed
!scroll begin speed_x 500 10000

// speed_y is the other SCROLL_SETTING
!scroll begin speed_y 100 1000

// You can also use the x and y shorthand
!scroll begin x 5 300
!scroll begin y 50 3000

// This is examples of stopping the scroll
!scroll begin x 0
!scroll begin y 0

!scroll primetime x 500
```

### !blur & !unblur

Blur or Unblur the source passed in.

!blur primetime 100

!move snoop 500 500

!scale snoop 200 200


```
!blur SOURCE AMOUNT DURATION_TO_REACH_BLUR_AMOUNT
```

If you send in the Blur amount to 0,  it's unbluring.
`!unblur SOURCE` and `!noblur SOURCE` do the same.

```
!blur begin 50
!blur begin
!blur begin 100 // This is the same as no value passed in
!unblur begin
!noblur begin
```

### !grow & !scale

Scale a Source using the actual OBS source settings, and no other filters

```
!scale SOURCE X Y
```

Scales the source's X and Y dimensions as a percentage of the total size. So 1.0
is the original size of the Object. `!scale SOURCE 0.5 0.5` is shrinking the
objects size by half for both the X & Y dimension.

```
!scale begin 0.5 0.5
!scale begin 1 1
!scale begin 1.1 0.2
```

### !3d

Requires the [Stream FX Plugin](https://github.com/Xaymar/obs-StreamFX)

Affect the 3D Transform Filter Settings.

**Format:**
```
!3d SOURCE FILTER_NAME FILTER_VALUE DURATION
```

**Effect:** Zoom in Close
```
!3d begin Scale.X 400
!3d begin Scale.Y 400
!3d begin Position.X -150

!3d begin Scale.X 100
!3d begin Scale.Y 100
!3d begin Position.X 0
```

**Effect:** Rotation a lot one way fast, then slowly the other way
```
!3d begin Rotation.Z 3600 3000
!3d begin Rotation.Z 0 90000
```

**Effect:** Hiding in the bottom and slowly Rising Up
```
// This hides you down below
!3d begin Position.Y 200 500

// This raises you slowly
!3d begin Position.Y 0 20000

```

---

### !norm

Return the source specified to normal. This is returning filters to normal, and
updating the size and position to more "normal" positions.

```
!norm begin
```

### !move

Move a source to a specific X & Y location.

```
!move SOURCE X Y
```

```
!move begin 500 500
```

### !tr / !tl / !br / !bl

Top-Right, Top-Left, Bottom-Right, Bottom-Left

```
!tr
!tl
!br
!bl
```

### !follow

Follow the source specified, by all sources added by viewers.

```
!follow garfield
```

### !staff

Figure the necessary filter and source move changes to not get banned by Staff in the chat.

There are no arguments to the `!staff` function.

```
!staff
```

### !create_source

Create a source in the Primary scene THAT ALREADY EXISTS

```
!create_source garfield
```

### !create_source_for_filters

Create the filters to manipulate a source, using StreamFX, SDF Effects and
others.

```
!create_source_for_filters garfield
```

### !filter

Print off information about a Filter.

```
!source FILTER_NAME
```

### !source

Print off information about a Source.

```
!source SOURCE
```

### !chat

Change to the Scene where you can read chat.

```
!chat
```

### !code

Change to the Scene where you can read code.

```
!code
```

### !memes !nomemes !nojokes !work


```
# Show the "memes" subscene
!memes

# Hide the "memes" subscene
# All aliases of the same thing
!nomemes
!nojokes
!work
```

### !ortho

!ortho technofroggo


### !show / !hide

!show a single source
!hide all sources


## Corner Pin

```
!corner frog Corners.BottomLeft.X 100
!corner frog Corners.BottomLeft.Y 10
!corner frog Corners.BottomRight.X 10
!corner frog Corners.BottomRight.Y 10
!corner frog Corners.TopLeft.X 10
!corner frog Corners.TopLeft.Y 10
!corner frog Corners.TopRight.X 10
!corner frog Corners.TopRight.Y 10

!corner frog Corners.BottomLeft.X 100
!corner frog Corners.BottomLeft.Y 10
!corner frog Corners.BottomRight.X 10
!corner frog Corners.BottomRight.Y 10
!corner frog Corners.TopLeft.X 10
!corner frog Corners.TopLeft.Y 10
!corner frog Corners.TopRight.X 10
!corner frog Corners.TopRight.Y 10
```

# Orthographic

```

!def_ortho frog
!ortho frog Scale.Y 1000
!ortho frog Shear.X 1000
!ortho frog Shear.Y 1000
!ortho frog Position.X -300
!ortho frog Position.X 0 3000
!ortho frog Position.Y 0
!ortho frog Rotation.X 36000
!ortho frog Rotation.Y 360
!ortho frog Rotation.Z 3600
```

## Perspective

```
!perp frog Camera.FieldOfView 1000
!perp frog Scale.X 200
!perp frog Scale.Y 200
!perp frog Shear.X 10
!perp frog Shear.Y 10
!perp frog Position.X 220
!perp frog Position.Y 220
!perp frog Position.Z 1
!perp frog Rotation.X 360
!perp frog Rotation.Y 360
!perp frog Rotation.Z 360

```

---

## Returning to Normal

- Right now Begin has Super+H to return as much as possible to normal

### Questions

- How does scroll stop???
