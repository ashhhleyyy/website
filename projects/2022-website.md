+++
title = "Website"
description = "My portfolio and blog website."
+++

This is the third version of my website, this time built using a custom [Rust](https://rust-lang.org/) backend. It uses [Askama](https://crates.io/crates/askama) templates, along with [axum](https://crates.io/crates/axum) for the HTTP server. It uses markdown for content, which rendered into HTML by [comrak](https://crates.io/crates/comrak) to place into the templates. My website functions fully without JavaScript, and it is only used to add a few (fairly obvious) easter eggs.

It's currently running on a single-node [kubernetes](https://kubernetes.io) cluster ([k3s](https://k3s.io) on [NixOS](https://nixos.org)) hosted in the cloud.

## Links

You can find my website in some of these places on the interwebs:
- [The site you are on now](https://ashhhleyyy.dev)
- [GitHub (source code)](github://ashhhleyyy/website)
- [Code mirror](https://git.ashhhleyyy.dev/mirror/website)
