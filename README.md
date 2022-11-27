# `subd`

`subd`'s goal is to create an interactive experience for viewers on any platform
and to support your supporters regardless of where they support you.

My goal is to have support for:
- Twitch viewers, subs, donators, memers,
- Github Sponsors
- Twitter supporters/shit-posters
- Youtube comment Warriors/Tik-Tok Comment Therapists
- `<more>`

And then to have integrations with:
- OBS
- Browser Source
- Twitch Chat+Moderation
- Discord
- Twitter
- Neovim
- Whatever we want!!!!

## Status

We are developing this on my twitch stream: [teej_dv](https://twitch.tv/teej_dv) and [beginbot](https://www.twitch.tv/beginbot)

## Development / Running

There are several steps required to set up `subd`. See [Developers](./DEVELOPERS.md) for more information.

// -------------------------------------------------------------

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
