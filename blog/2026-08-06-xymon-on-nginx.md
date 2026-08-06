+++
title = "Hosting Xymon monitoring with nginx"
description = "Deploying my favourite monitoring service without Apache HTTPd"
+++

I love [Xymon](https://xymon.sourceforge.io/). It's a monitoring system for servers, as well as just about anything you can shove on a network, and unlink things like Grafana+Prometheus+AlertManager, you don't need a whole stack of applications for alerting+metrics+a status page.

<!-- <fedi-post data-server="fedi.shorks.gay" data-id="amjm3inswl9wqyar"></fedi-post> -->

## Why?

Xymon is great for monitoring a collection of servers and other network infrastructure and devices. At my student radio station, we used it to monitor our rack full of servers, as well as our router, switches, and other studio equipment. Xymon is primarily aimed at monitoring hosts rather than applications, so if you're running something like Kubernetes, you would probably want something separate to perform healthchecks on your applications, but you could still use Xymon to monitor your cluster nodes and ensure that services such as the kubelet and any external databases are running.

With things like [Devmon](https://github.com/bonomani/devmon), you can also extend it to support pulling data from SNMP-enabled devices, such as routers and network switches. You can also just write scripts on your hosts to check the health of anything Xymon doesn't have built-in support for. For example, I use [this script](https://wiki.xymonton.org/doku.php/monitors:bb-zfs) to monitor the ZFS pools on our filestore and backup servers.

<dialogue character="leah" mood="happy">
    So you run Xymon on your own infrastructure then?
</dialogue>

Unfortunately, not yet.

## Downsides

<dialogue character="leah" mood="surprised">
    ah... of course
</dialogue>

While I love Xymon, it isn't really packaged for many distros. Debian package both [the server](https://packages.debian.org/trixie/xymon) and [the client](https://packages.debian.org/trixie/xymon-client), along with a bunch of [useful scripts](https://packages.debian.org/trixie/hobbit-plugins) for monitoring things like postgres and apt updates<fn content="If you're wondering why the package is called hobbit-plugins; Xymon used to be called Hobbit, and is a reimplementation of a previous tool called Big Brother"></fn>. FreeBSD also packages all of Xymon, however neither package other tools like Devmon, and there is only a [non-nixpkgs NixOS module for the client](https://github.com/daduke/xymon-client-nix).

Packaging the server for Nix is something I am already working on, however this post is (as well as just sharing my love of xymon) primarily about the other issue: most documentation only includes information on serving the web UI with Apache2, whereas I prefer using nginx.

<dialogue character="leah" mood="happy">
    Couldn't you just, y'know, run Apache on a different port, and reverse proxy it from nginx.
</dialogue>

...

Well yes, but I would prefer not to have so much web server on my web server.

## NGINX time

I'm assuming you've already got the `xymon` and `nginx` packages installed on a debian machine, but as Xymon's is a CGI application, you'll also want `fcgiwrap` installed, which allows running regular CGI binaries and scripts using FastCGI.

First, create a snippet for routing requests to fcgiwrap (`/etc/nginx/fcgiwrap`):

```
location ~ ^/.*\.sh$ {
    gzip off;
    fastcgi_param SCRIPT_FILENAME $request_filename;
    fastcgi_param REMOTE_USER $remote_user;
    include fastcgi_params;
    fastcgi_pass unix:/var/run/fcgiwrap.socket;
}
```

Then, add the following blocks in the `server {}` you want to host xymon out of. This will serve the xymon UI at /xymon/:

```
rewrite ^/xymon$ /xymon/;

location /xymon/ {
    auth_basic "Xymon";
    auth_basic_user_file /etc/nginx/xymon.htpasswd;
    alias /var/lib/xymon/www/;
}

location /xymon-cgi/ {
    auth_basic "Xymon";
    auth_basic_user_file /etc/nginx/xymon.htpasswd;
    alias /usr/lib/xymon/cgi-bin/;
    include fcgiwrap;
}

location /xymon-seccgi/ {
    auth_basic "Xymon";
    auth_basic_user_file /etc/nginx/xymon.htpasswd;
    alias /usr/lib/xymon/cgi-secure/;
    include fcgiwrap;
}
```

Feel free to replace the `auth_basic` with whatever you usually use for authentication, but you may need to add handling for passing the authenticated username to Xymon, as it is used for things like recording who disabled an alert.

To create users, I use the `htpasswd` tool from `apache2-utils`:

```bash
# For the initial user:
sudo htpasswd -c -B /etc/nginx/xymon.htpasswd <username>
# To add more users:
sudo htpasswd -B /etc/nginx/xymon.htpasswd <username>
```

Hope this is helpful to some people, and I'll see if I can write some more posts about Xymon stuff soon!

<footnotes></footnotes>
