+++
title = "From Keycloak to Kanidm"
description = "Or, \"Why I should just move my server to NixOS\""
+++

Or, "Why I should just move my server to [NixOS](https://nixos.org)".

[Kanidm](https://github.com/kanidm/kanidm#readme) is The Hot New Thing (alright, its not actually that new, and it's still technically in alpha, but still), and I wanted to replace my resource-heavy [Keycloak](https://keycloak.org) server with it, to hopefully free up my server for more ~~important~~ tasks.

## Docker makes things 'easy'

Kanidm [provides prebuilt docker images](https://hub.docker.com/r/kanidm/server) for the server, and this was fairly easy to get running:

```yaml
kanidm:
  docker_container.running:
    - name: kanidm-main
    - image: kanidm/server:latest
    - restart_policy: on-failure:5
    - binds:
      - /mnt/data/services/kanidm:/data
      - /etc/letsencrypt:/certs:ro
    - port_bindings:
      - 8443:8443
      - 3636:3636
```

All I had to do was make a configuration file like so (lots of stuff from the default config generator skipped):

```toml
# Address to bind the HTTPS server
bindaddress = "[::]:8443"
# Address to bind the LDAPS server
ldapbindaddress = "[::]:3636"
# I use nginx as a reverse proxy, and set this header accordingly
trust_x_forward_for = true
# Default for the docker setup
db_path = "/data/kanidm.db"
# Managed my certbot on the host
tls_chain = "/certs/live/ashhhleyyy.dev/fullchain.pem"
tls_key = "/certs/live/ashhhleyyy.dev/privkey.pem"
# Lots of logging
log_level = "verbose"

domain = "sso.ashhhleyyy.dev"
origin = "https://sso.ashhhleyyy.dev"
```

However, due to weird issues with the container builds, the `latest` tag isn't the most recent version, and the `x86_64_latest` is only compatible with `x86_64` CPUs, but my Raspberry Pi 4 has an `arm64` CPU. This means I have to use the slightly-outdated `latest` tag (this will cause issues later).

## Configuring some clients

Next up is to configure all my existing things to use Kanidm, which is a bit of a repetitive task, although I prefer the copy-paste command line configuration for Kanidm over Keycloak's web-based admin portal.

First up, I have Vouch set up to [secure certain pages](/blog/2022-09-22-programming-at-school-2) only I should be able to access, and so first I need to create an oauth2 app for it:

```
$ kanidm login --name admin
Enter password:
Login Success for admin

$ kanidm system oauth2 create vouch "Internal Services" https://vouch.ashhhleyyy.dev
Success

$ kanidm system oauth2 show_basic_secret vouch
[REDACTED]
```

Cool! Now I can update the configuration for Vouch and restart the service:

```yaml
# previous sections skipped

oauth:
  provider: oidc

  # kanidm
  client_id: vouch
  client_secret: <REDACTED>
  auth_url: "https://sso.ashhhleyyy.dev/ui/oauth2"
  token_url: "https://sso.ashhhleyyy.dev/oauth2/token"
  user_info_url: "https://sso.ashhhleyyy.dev/oauth2/openid/vouch/userinfo"

  code_challenge_method: S256

  scopes:
    - email
  callback_url: https://vouch.ashhhleyyy.dev/auth
```

```
$ docker restart vouch-main
vouch-main
$ 
```

Aaand done! Let's try it out...

![Ferris the rustacean, above a dialog that reads "Access Denied. You do not have access to the requested resources."](/assets/images/from-keycloak-to-kanidm/access-denied.png)

Wait...

It doesn't work :/

## The scope of the issue

Kanidm has a feature called "scope maps", which controls which users/clients are able to request certain scopes when logging in. For OpenID Connect, a scope named `openid` [MUST](https://www.rfc-editor.org/rfc/rfc2119) be requested, along with additional scopes which control what user information is included in the provided token, such as [`profile`, `email` or `address`](https://github.com/kanidm/kanidm/blob/master/kanidm_book/src/integrations/oauth2.md#:~:text=HINT%20OpenID,connect).

Vouch only needs the `openid` and `email` scopes to function, so we just need to create a scope mapping which grants those:

```
$ kanidm system oauth2 update-scope-map gitea idm_all_persons openid email
Success
```

...

![Ferris the rustacean, above another dialog that reads "Consent to proceed to Internal Services. This site will not have access to your personal information"](/assets/images/from-keycloak-to-kanidm/approved.png)

Woo!

This should be enough to allow any user (`idm_all_persons`) to authenticate with Vouch, which should be enough for a single-user setup where the goal is simply to unify login. However, we need to make some changes if we'd like to create accounts for other people and control what they have access to...

## Groups, groups, groups!

We've already used a group in the previous part, `idm_all_persons`, which automatically includes all "person" accounts (which are different to service accounts like `admin` and `idm_admin`), but we can also create our own and assign users to them.

First, lets create a group for users which should be able to access Vouch-protected resources:

```
$ kanidm group create vouch-access
Successfully created group 'vouch-access'
```

Now, lets add a user to the new group (change `ash` for whatever your username is)

```
$ kanidm group add_members vouch-access ash
Successfully added ["ash"] to group "vouch-access"
```

Finally, we need to remove the old scope map, and create a new one:

```
$ kanidm system oauth2 delete_scope_map vouch idm_all_persons
Success

$ kanidm system oauth2 update-scope-map vouch vouch-access openid email
Success
```

Now this still works for my main account, `ash`, but I get an "Access denied" error if I try using `ash2`, who is not a member of the `vouch-access` group:

<!-- Yes I'm reusing the same screenshot, no one is stopping me :P -->
![Uh oh, Ferris says we aren't allowed again](/assets/images/from-keycloak-to-kanidm/access-denied.png)

This technique of using groups to control access to different services is something that I couldn't easily figure out how to do with Keycloak, and so its great that this is such an obvious feature in Kanidm!

## Migrating some other services

I use [Grafana](https://grafana.com) for making pretty graphs of things, and moving it over to use Kanidm was basically the same as with Vouch - create an oauth2 application, add a group, and update the Grafana configuration file:

```ini
[auth.generic_oauth]
enabled = true
name = Kanidm
client_id = grafana
client_secret = [REDACTED]
scopes = openid email profile
login_attribute_path = prefered_username
auth_url = https://sso.ashhhleyyy.dev/ui/oauth2
token_url = https://sso.ashhhleyyy.dev/oauth2/token
api_url = https://sso.ashhhleyyy.dev/oauth2/openid/grafana/userinfo
```

Other services, however, did not want to play as nicely with Kanidm...

## Git-eAAAAA

My [Forgejo](https://forgejo.org/) instance was the next thing I wanted to move over to Kanidm, and so I started out the same way as before, but quickly hit a roadblock: usernames.

![Forgejo's 500 internal server error page](/assets/images/from-keycloak-to-kanidm/forgejo-error-500.png)

Uhhh.

```
2023/02/06 12:26:51 ...ers/web/auth/auth.go:570:createUserInContext() [E] [63e0f20a] CreateUser: name is invalid [ash@sso.ashhhleyyy.dev]: must be valid alpha or numeric or dash(-_) or dot characters
```

Ah.

After a bit of digging, I realised Kanidm can be configured per-oauth2 client to prefer either an "SPN" or just the short username in the `prefered_username` field of the token, and so a quick

```
$ kanidm system oauth2 prefer-short-username forgejo
Success
```

later, and everything should be working now.

![The same 500 internal server error page, I really can't be bothered to take two different screenshots of the same message](/assets/images/from-keycloak-to-kanidm/forgejo-error-500.png)

Hmmmmmmmmm

Eventually, after a lot of digging, I figured out what the issue was:

<fedi-post data-server="fedi.ashhhleyyy.dev" data-id="AS10mMiPosXdgjUKv2"></fedi-post>

In Kanidm v1.1.0-alpha.10, when `prefer-short-username` was added, the implementation did not take into account the [OpenID Connect userinfo endpoint](https://openid.net/specs/openid-connect-core-1_0.html#UserInfo), which is used by many implementations to fetch the full details about the authenticated user.

The solution to this should simply be to update to the latest version (which was released while I was trying to work out what was going wrong), however the `arm64` images fail to build properly, due to missing prebuilt binaries for [`wasm-opt`](https://github.com/WebAssembly/binaryen#tools). As of writing, [this PR](https://github.com/rustwasm/wasm-pack/pull/1102) for [`wasm-pack`](https://github.com/rustwasm/wasm-pack/) to fix this problem has not been merged.

This meant I had to adjust Kanidm's Dockerfile manually to either provide the required `wasm-opt` binary, remove the optimisation pass (ideally not), or use a prebuilt WASM binary for the web UI. I choose the first option, and (after learning how to use alternative package repositories with [`zypper`](https://en.opensuse.org/Portal:Zypper)), I came up with the following required changes:

```patch
diff --git a/kanidmd/Dockerfile b/kanidmd/Dockerfile
index 0f0125a7f..17feefe2c 100644
--- a/kanidmd/Dockerfile
+++ b/kanidmd/Dockerfile
@@ -1,6 +1,7 @@
 ARG BASE_IMAGE=opensuse/tumbleweed:latest
 FROM ${BASE_IMAGE} AS repos
-RUN zypper refresh --force
+RUN curl -o '/etc/zypp/repos.d/home:dziobian:gulgul-ultron.repo' 'https://download.opensuse.org/repositories/home:/dziobian:/gulgul-ultron/openSUSE_Tumbleweed/home:dziobian:gulgul-ultron.repo'
+RUN zypper --gpg-auto-import-keys refresh --force
 RUN zypper dup -y
 
 # ======================
@@ -17,6 +18,8 @@ RUN zypper install -y \
         rsync \
         findutils \
         which
+RUN zypper install -y --from home_dziobian_gulgul-ultron \
+        binaryen
 RUN zypper clean -a
 RUN rustup default stable
 
```

([full patch with some Makefile changes too](https://git.ashhhleyyy.dev/ash/e/src/branch/main/patches/kanidm-docker-fix.patch))

Finally, I can build and push all the Kanidm images with `make buildx`, which will use BuildKit to cross-compile from my PC to both 'standard' x86_64, and arm64. You can also use some alternative `make` targets like `buildx/kanidmd`, `buildx/kanidm_tools` and `buildx/radiusd` to only build singular components. For me this took quite a long time (in hours), because the `arm64` build happens inside a QEMU emulation layer, which significantly slows down the CPU-intensive compilation process.

I chose to just build `kanidmd` (the main server) and `kanidm_tools` (the command-line client), as right now I don't need a [RADIUS](https://en.wikipedia.org/wiki/RADIUS) server, and so I used the following command:

```
$ env IMAGE_BASE=git.ashhhleyyy.dev/ash make buildx/kanidmd buildx/kanidm_tools
```

This built and pushed the two images, which are compatible with both `x86_64` and `arm64` 🎉🎉🎉

> If you'd like to use these prebuilt images, they're available on my Forgejo [here (kanidm_tools)](https://git.ashhhleyyy.dev/ash/-/packages/container/kanidm-tools/devel) and [here (kanidmd)](https://git.ashhhleyyy.dev/ash/-/packages/container/kanidm-server/devel), but I've also provided everything needed to build them from source too :)
>
> Of course, if you're on `x86_64`, you probably can just use the official images [on Docker Hub](https://hub.docker.com/r/kanidm/server).

---

## Wrapping up

Finally, once I had updated Kanidm, Forgejo basically Just Works&trade;, and I can continue moving services over, all of which worked without any hitch. I've kept my old Keycloak instance running for now, in case I've missed anything that still depends on it, however I've disabled all the clients that I have moved over, and I'm hoping I can stop running the server in the next few weeks and nothing will break :)
