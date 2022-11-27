# Commands

The commands available from Twitch Chat, what they do, and how they work.

## !scroll

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
```

## !blur & !unblur

Blur or Unblur the source passed in.

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

## !grow & !scale

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


## !3d

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

## !norm

```
!norm begin
```

## !move

Move a source to a specific X & Y location.

## !tr / !tl / !br / !bl

Top-Right, Top-Left, Bottom-Right, Bottom-Left

## !follow

Follow the source specified, by all sources added by viewers.

## !staff

Figure the necessary filter and source move changes to not get banned by Staff in the chat.

## !create_source

Create a source in the Primary scene THAT ALREADY EXISTS

## !create_source_for_filters

Create the filters to manipulate a source, using StreamFX, SDF Effects and
others.

## !filter

Print off information about a Filter.

## !source

Print off information about a Source.

## !chat

Change to the Scene where you can read chat.

## !code

Change to the Scene where you can read code.
