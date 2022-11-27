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
