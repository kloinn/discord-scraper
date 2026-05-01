# discord-scraper

Snapshot of Aconite's private respository, which is a Discord scraper I was developing for fun years ago. Back then, I had no idea what I was doing and quite frankly, and I didn't even know the difference between .clone() and a reference, and didn't know about .unwrap(). 

This codebase is terrible. Only use this as a reference (and it also flags Discord's anti-spam filters)

## CSAM & NSFW

This repository contains NSFW & CSAM blocking logic, both of which are relying on specific internal services that won't ever be made public. 

The CSAM handling logic is in `automod.rs`, while NSFW is in `image.rs`.

## DSA

This repository also has DSA auto-reporting logic, but it is unfinished and doesn't work.

### Email

This repository also contains email-related logic for receiving DSA emails, specifically in `email.rs`, which also uses Gmail under the hood over Duckduckgo's email forwarder. 

This will cause this scraper to crash if you don't provide a `clientsecret.json` file, however since email is not a required feature, you can just comment all of its logic out.

This also has a hardcoded email of `discordreporting@duck.com` (https://github.com/kloinn/discord-scraper/blob/main/src/email.rs#L72, https://github.com/kloinn/discord-scraper/blob/main/src/dsa.rs#L142), which should be changed or commented out.

### Cloudflare

This uses a stale __dcfduid cookie which is known for flagging Discord's spam filters (confirmed by the Zendesk Discord breach) (https://github.com/kloinn/discord-scraper/blob/main/src/dsa.rs#L67), aswell as a hardcoded timezone in https://github.com/kloinn/discord-scraper/blob/main/src/dsa.rs#L63
