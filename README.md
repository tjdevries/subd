# `subd`

`subd`'s goal is to create an interactive experience for viewers on any platform
and to support your supporters regardless of where they support you.

My goal is to have support for:
- Twitch subs
- Github Sponsors
- `<more>`

And then to have integrations with:
- OBS
- Browser Source
- Twitch Chat
- Twitch Moderation
- (don't see any reason why it shouldn't be possible to do for YT)
- Neovim
- who knows :)

## Status

We are developing this on my twitch stream: [teej_dv](https://twitch.tv/teej_dv)

Currently, we are building the different building blocks we need to start creating interesting information.
Not much to see our try out here at this time.

## Development / Running

There are several steps requied to set up `subd`. See [Developers](./DEVELOPERS.md) for more information.

## OBS Setup

### Creating a new Scene

- Create the Scene is Manually
- create filters

```
!create_filters_for_source INSERT_SOURCE_NAME
!create_filters_for_source kidalex
```

This will create X Number of Filters:
    - Blur
    - Scroll
    - 3D Transform
    - SDF Effects
    - Move-Value Filters for each of those move
    - Move-Value to Defaults filters
    - Move Source on "Primary" scene
