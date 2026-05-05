# deadlocked

[![Matrix Invite](https://img.shields.io/matrix/open-source-cs2-hacking%3Amatrix.org?style=for-the-badge&logo=matrix&label=Matrix)](https://matrix.to/#/%23open-source-cs2-hacking:matrix.org)

[![Discord Invite](https://img.shields.io/discord/1333541580249890949?style=for-the-badge&logo=discord&logoColor=white&label=Discord)](https://discord.gg/eXjG4Ar9Sx)

[![Casual Maintenance Intended](https://casuallymaintained.tech/badge.svg)](https://casuallymaintained.tech/)

simple cs2 aimbot and esp, for linux only.

## Setup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/avitran0/deadlocked
cd deadlocked
./setup.sh
# Restart your machine (required)
```

Also make sure that the `uinput` kernel module is loaded.

Running NixOS or Fedora Atomic? See [OS-Specific Setup](os-setup.md).

## Running

```bash
./run.sh
```

## Features

### Aimbot

- Enable/disable and hotkey (hold or toggle)
- Does not run on knife, grenades, or unknown weapon class (still runs on pistols and shotguns, unlike RCS)
- FOV limit and smoothing; optional **distance-adjusted FOV** (tighter effective FOV for farther targets)
- Targeting mode: pick the enemy with the smallest on-screen angle to the **head** inside the FOV cone, or the closest by **world distance** (still FOV-gated)
- After a target is chosen, the aim point is whichever **bone in your configured list** has the smallest on-screen angular error (the list is a whitelist, not a priority chain)
- Start bullet: only apply aim once `shots_fired` reaches N in the current burst (0 = including the first shot)
- Visibility check using the loaded map mesh (BVH): line-of-sight is tested from your eye to several sample bones (head, hands, feet). If no mesh is loaded for the map, it falls back to the engine spotted mask
- Flash check: skip when you are blinded
- Optional target friendlies (skipped by default; FFA is detected so mixed-team modes still work)
- Per-**weapon** overrides (each gun entry, e.g. AK-47 vs AWP), not a coarse “rifle vs SMG” bucket
- Optional FOV circle on the overlay when aimbot is active; with distance-adjusted FOV, **three** reference rings are drawn at 125 / 250 / 500 hammer units (green / yellow / red)

### Triggerbot

- Enable/disable and hotkey (hold or toggle)
- Fires on the **crosshair entity** returned by the game (no separate BVH/spotted visibility gate like the aimbot)
- Skips teammates unless the match is treated as FFA
- Delay: each shot samples a delay in ms from a normal distribution centered between your min and max (spread is half the min–max span)
- Shot duration: how long the left mouse button stays down per trigger (via uinput)
- Flash, scope (snipers must be scoped), and local **velocity** checks with configurable speed threshold
- Head-only: only fires if the crosshair is within an approximate angular radius of the target’s head

### Standalone RCS

- Separate X/Y strength (0–1); runs in the game loop before aimbot
- **Disabled** on pistols, shotguns, knife, grenades, and unknown class; enabled on rifles, SMGs, LMGs, snipers, etc.
- Snipers: special handling when aim punch reads as zero mid-burst
- Optional per-weapon overrides (same model as aimbot)

### Per-weapon overrides

For each CS2 weapon slot you can enable an override and supply separate aimbot, triggerbot, and RCS blocks; the UI uses a Global profile plus a per-weapon tab.

### Player (ESP)

- ESP hotkey and master enable
- Show friendlies (when not in teammate-vs-teammate FFA)
- Visible-only filter (hide ESP when the cheat considers the player occluded / unspotted)
- Box coloring: off, by health, or visible vs not visible (custom visible / invisible colors)
- Box outline: 2D gap corners, 2D full rectangle, or **3D** yaw-oriented world wireframe from feet to head (3D is exclusive of the 2D box modes)
- Box fill: none, solid, or vertical gradient (2D full box only; alpha slider for fill strength)
- Distance in meters under the label stack (~52.49 hammer units per meter, approximate)
- Optional **SCOPE** / **FLASH** status text when the pawn reports ADS or flash duration
- Skeleton with the same color modes as the box; optional head circle
- Health and armor bars; name; weapon icon and magazine text (`current/max` when available); icon tags for helmet, defuser, bomb
- Sound ESP: optional emphasis when the player is classified as making footstep, gunshot, or scoped-weapon noise within configurable diameters, with fade-in / fade-out timing; optional **always full alpha** for players already considered visible so sound emphasis does not dim them

### HUD

- Bomb timer: world text, bottom screen bar, and defuse countdown text when a defuse is in progress
- Dropped weapons in the world
- Grenade flight trails with per-grenade-type colors (smoke, molotov, incendiary, flash, HE, decoy)
- Map grenade lineup editor: lineups are stored in `grenades.json` next to your profiles; saved spots draw a world polygon and an on-aim panel when you are close, on the right grenade, and match saved jump/duck/run modifiers (see **Grenades** tab)
- Sniper overlay crosshair (length, gap, line width, color)
- FOV circle when aimbot is eligible (respects hold vs toggle state)
- Keybind list and spectator list
- Debug overlay: full-screen corner cross lines for checking overlay alignment
- Vote HUD: YES/NO or generic vote counts while `C_VoteController` reports an active issue (depends on schema-based offset resolution)
- Hit feedback: crosshair ticks plus a brief damage-number popup when your tracked round-dealt damage increases (`m_flTotalRoundDamageDealt`); optional beep via the default output device (rodio)

### Application / UI

- Egui menu with tabs: Aimbot, Player, Hud, Grenades, Unsafe, Config, Application — plus About and Report Issue shortcuts in the sidebar
- Menu accent color, configurable overlay FPS cap, HUD text color/outline, font sizes, line width, icon size
- Profiles as `.toml` files under `configs/` in the config directory (see FAQ)
- First-launch prompt for optional anonymous crash stack traces

### Unsafe

> [!WARNING]
> These features write to game memory and might get you banned.

- No flash: clamps maximum flash alpha (configurable ceiling)
- FOV changer: continuously writes desired FOV to the local controller (`m_iDesiredFOV`, clamped)
- No smoke / smoke suppression
- Optional smoke particle tint

## FAQ

### Where are my configs saved?

Configs are saved in `$XDG_CONFIG_HOME` with fallback to `$HOME/.config`. Otherwise they're saved alongside the executable.

### Which desktop environments and window managers are supported?

**Best support:**

- GNOME (Mutter)
- KDE (KWin)

**Good support:**

- SwayWM
- Weston

**Fair support:**

- i3
- OpenBox
- XFCE
- Hyprland (tweaks may be needed; no guarantees)

### I'm using Hyprland and something doesn't work

Hyprland has poor X11 support for the techniques this cheat uses, not much i can do about that.
Try another WM if possible.

### I'm using Gamescope and the overlay is too small

The game still thinks it's running in 16:9 resolution, so the cheat gets the wrong window resolution.
Try running the game without Gamescope.

### My screen/overlay is black

Your compositor or window manager doesn't support transparency, or it's not enabled.

On KDE, go into the `Display and Monitor` settings, then `Compositor`, and tick `Enable compositor on startup`.

### The overlay shows but I can't click anything

The window couldn't be made click-through. This is a window manager/compositor limitation.

### The overlay doesn't show up

Your window manager doesn't support positioning or resizing windows.

### The overlay isn't on top of other windows

Your window manager doesn't support always-on-top windows.
