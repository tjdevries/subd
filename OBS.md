# OBS

## Move Value Filters

This is for Move Value Type: "Settings"

```
"settings": {
    "duration": 7000,
    "filter": "Blur",
    "move_value_type": 0,
    "setting_float": 0.0,
    "setting_float_max": 0.0,
    "setting_float_min": 0.0,
    "setting_name": "",
    "value_type": 0
},
```

This is for an Experimental Blur

This is while it's on "Settings"
"move_value_type": 1

Notice: It just adds the field and amount to the JSON
in this case "Filter.Blur.Size": 100.0

```json
{
    "Filter.Blur.Size": 100.0,
    "custom_duration": true,
    "duration": 6969,
    "filter": "Blur",
    "move_value_type": 1,
    "value_type": 0
}
```

Now trying for Single Setting
"move_value_type": 0,
So 0 corresponds to single-setting in this case
Also note, it now has 3 values:
    -> Min and max for float
    -> and the actual value

    -> As well as the setting_name

The min and max are for the random I'm pretty sure

```json
{
    "Filter.Blur.Size": 100.0,
    "custom_duration": true,
    "duration": 6968,
    "filter": "Blur",
    "move_value_type": 0,
    "setting_float": 51.0,
    "setting_float_max": 100.0,
    "setting_float_min": 100.0,
    "setting_name": "Filter.Blur.Size",
    "value_type": 2
}
```

## Random

"move_value_type": 2
this is for random

```json
{
    "Filter.Blur.Size": 100.0,
    "custom_duration": true,
    "duration": 511,
    "filter": "Blur",
    "move_value_type": 4,
    "setting_float": -10.0,
    "setting_float_max": -10.0,
    "setting_float_min": -10.0,
    "setting_name": "Filter.Blur.Size",
    "value_type": 2
},
```
## Add

"move_value_type": 3,
So this is for Add

WHAT IS VALUE TYPE HERE???
    "value_type": 2
It's a negative number does that do anything?
float???
```json
{
    "Filter.Blur.Size": 100.0,
    "custom_duration": true,
    "duration": 511,
    "filter": "Blur",
    "move_value_type": 3,
    "setting_float": -10.0,
    "setting_float_max": 100.0,
    "setting_float_min": 1.0,
    "setting_name": "Filter.Blur.Size",
    "value_type": 2
},
```

---

## Simplified Move Value Settings

## Base Settings

```json
{
    "move_value_type": 0,
    "setting_float": 50.0,
    "setting_name": "Filter.Blur.Size",
    "value_type": 0
},
```

## Single Setting

```json
{
    "move_value_type": 0,
    "setting_float": 50.0,
    "setting_name": "Filter.Blur.Size",
    "value_type": 0
},
```

## Multiple Settings

```json
{
    "move_value_type": 1,
    "Filter.Blur.Size": 100.0,
    "value_type": 0
}
```

## Random

I feel like this add
```json
{
    "move_value_type": 2,
    "filter": "3D Transform",
    "setting_float_max": 90.0,
    "setting_float_min": 9.0,
    "setting_name": "Camera.FieldOfView",
    "value_type": 2
}
```

## Add

```json
{
    "move_value_type": 3,
    "setting_float": -10.0,
    "setting_name": "Filter.Blur.Size",
    "value_type": 2
}
```

## Typing

We need to explore this a little more


```json
{
    "easing_match": 0,
    "setting_decimals": 1,

    "setting_float": 1.0,
    "setting_float_max": 0.0,
    "setting_float_min": 0.0,
    "setting_int": 0,
    "setting_int_max": 0,
    "setting_int_min": 0,

    "move_value_type": 4,
    "setting_name": "text",
    "setting_text": "Ok NOW",
    "value_type": 4
}
```
