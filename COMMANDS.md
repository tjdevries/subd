# Commands

The commands available from Twitch Chat, what they do, and how they work.

## !scroll

```
!scroll SOURCE SCROLL_SETTING speed duration
```

Scroll Settings:
    - speed_x
    - speed_y

Real Examples:
```
// Scroll the begin source
// along x at a speed of 500,
// taking 10000 milliseconds to reach full speed
!scroll begin speed_x 500 10000
!scroll begin speed_y 100 1000

// You can also use the x and y shorthand
!scroll begin x 5 300
!scroll begin y 50 3000
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
