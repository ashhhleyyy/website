+++
title = "How I program at school"
description = "A speedy guide to coding on (relatively) locked down computers."
+++

A speedy guide to coding on (relatively) locked-down computers.

## Context

I am currently doing my A-Levels in the UK, and the computers at my school (like most others) are quite restrictive in what can be run on them and what tools are available. While they include Python and its built-in IDE, IDLE, there is not much else in the way of programming, such as tools like [Visual Studio Code](https://code.visualstudio.com/), and so I usually use a cloud based IDE.

## Repl.it

When I started this year, I began by using [repl.it](https://replit.com) to do programming while in school. This was OK for a bit, but I quickly found the limitations. Repl.it did not provide full IDE-like support for some languages, mainly Rust (which is what I enjoy the most). I also found that the VMs/containers/whatever that they use to run workspaces are quite low-powered and can struggle with more intensive tasks.

## Gitpod

At some point, I discovered [Gitpod](https://gitpod.io/), which provides Docker-based containers running VS Code and whatever other tools you want, and runs, like repl.it, entirely in the browser. This is what I used for most of the year, working inside a private monorepo with all my projects, using several languages and tools, including Rust, Vite.js and Go. I would also occasionally create separate repositories for certain projects, when I felt like they should be kept apart.

## OpenVSCode-Server

This is actually a [project from the team behind Gitpod](https://github.com/gitpod-io/openvscode-server), and provides

> A version of VS Code that runs a server on a remote machine and allows access through a modern web browser.

I can run this on both the [Raspberry Pi Zero](https://raspberrypi.com/) that I carry at school, or on my PC at home, which allows me to work on projects that I have locally and work on at home, without having to context-switch to the browser based Gitpod client. My setup for this is quite convoluted, but is essentially compromised of the following steps:

1. Connect to my PC at home via my Raspberry Pi at school using [Tailscale](https://tailscale.com/)
2. Run OpenVSCode-Server on my PC inside a `tmux` session
3. Use SSH to port-forward the VSCode server, along with port 3000, which I usually use for webdev stuff:
```bash
ssh -L 0.0.0.0:7070:localhost:7070 -L 0.0.0.0:3000:localhost:3000 ash-pc
```
4. Open VSCode in the browser.

I also have my local VSCode `settings.json` symlinked to the location of the one for the OpenVSCode-Server, so that my settings are synchronised across both, and then I am able to work on my projects at school almost exactly as I do at home!
