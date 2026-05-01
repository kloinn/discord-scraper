# discord-scraper

Snapshot of Aconite's private respository, which is a Discord scraper I was developing for fun years ago, quite frankly, back then I didn't even know the difference between .clone() and a reference. 

This codebase is terrible. Only use this as a reference (and it also flags Discord's anti-spam filters)

## CSAM & NSFW

This repository contains NSFW & CSAM blocking logic, both of which are relying on specific internal services that won't ever be made public. 

The CSAM handling logic is in `automod.rs`, while NSFW is in `image.rs`.

## DSA

This repository also has DSA auto-reporting logic, but it is unfinished and doesn't work.
