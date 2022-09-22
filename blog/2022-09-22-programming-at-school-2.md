+++
title = "Improving my experience programming at school"
description = "Several changes to my setup for programming from school computers"
+++

It's a new school year for me, so that means that I've been making several improvements to my setup for programming from school!

Firstly, like [last time](/blog/2022-05-12-programming-at-school), I'm still using [OpenVSCode Server](/blog/2022-05-12-programming-at-school#openvscode-server) as an IDE. However, I have made a few changes to how I access it, mainly the removal of the Raspberry Pi proxy on the school network.

## (Ab)using nginx

While setting my home Raspberry Pi server up again from scratch after re-installing, I came up with an idea inspired by the way [Gitpod](https://gitpod.io/) handles forwarding ports from within a workspace.

The way they do it is by placing the port number in the hostname, which their reverse proxy (`caddy`) extracts and then forwards onwards to the correct port in the workspace container, and so I thought it would be interesting to see if I could achieve this using `nginx` instead. I came up with the following nginx configuration block:

```
map $http_upgrade $connection_upgrade {
	default upgrade;
	'' close;
}

server {
	# Standard HTTPS stuff
	listen 443 ssl;
	listen [::]:443 ssl;

	ssl_certificate /etc/letsencrypt/live/ashhhleyyy.dev/fullchain.pem;
	ssl_certificate_key /etc/letsencrypt/live/ashhhleyyy.dev/privkey.pem;

	# Parse the hostname using a (messy) regex
	server_name "~^(?<port>[0-9]{1,5})-internal\.ashhhleyyy\.dev$";

	location / {
		# Pass the request on to my computer
		proxy_pass http://<TAILSCALE IP YEETED>:$port;
		# Allow websocket connections to pass through correctly
		proxy_http_version 1.1;
		proxy_set_header Upgrade $http_upgrade;
		proxy_set_header Connection $connection_upgrade;
		proxy_set_header Host $host;
	}
}
```

This forwards domains like `3000-internal.ashhhleyyy.dev` to `<my PC>:3000`, which allows me to access things like OpenVSCode Server or a development server for whatever project I'm working on in the browser. This also requires a wildcard DNS record pointing to the server, which I have anyway to allow setting up new subdomains easier.

Looking at this you might be thinking "but surely anyone could access your stuff now", which is what I thought, and so I started looking for something to add authentication.

## HTTP Basic auth

A simple approach to protect this would be to make use of nginx's built-in support for HTTP Basic Authentication, which just takes a `username:password` pair and looks it up in an `/etc/passwd`-style file.

However, I run an instance of [Keycloak](https://keycloak.org/) for easy sign in for some other things already (mainly my [Gitea](https://git.ashhhleyyy.dev/) and [Grafana](https://grafana.com) instances), so this led me to look for a way to integrate that instead...

## Vouch-proxy

I came across [vouch-proxy](https://github.com/vouch/vouch-proxy), which makes use of nginx's [auth_request](http://nginx.org/en/docs/http/ngx_http_auth_request_module.html) module to allow single sign-on (SSO) using OpenID Connect, and it's exactly what I was looking for. Setting it up was really painless, I first created a client for it on my Keycloak realm:

![Creating a client named "Vouch" on Keycloak's administration console](/assets/images/programming-at-school-2/vouch-client.png)

I then had to add turn on the "Client authentication" toggle to show the Credentials tab where the OAuth client secret can be found:

![A section of the Keycloak client settings, showing a toggle named "Client authentication" set to on](/assets/images/programming-at-school-2/vouch-client-authentication.png)

To run Vouch, I used Docker, as that's what I use for most other services on my Pi, and so I created a configuration file that I can then mount into the container:

```yml
vouch:
  # Allowlist of domains vouch will set a cookie on
  domains:
  - ashhhleyyy.dev

  # Make the cookie HTTPS only
  cookie:
    secure: true

oauth:
  # Generic OpenID Connect
  provider: oidc
  client_id: {{ pillar["vouch"]["keycloakClientId"] }}
  client_secret: {{ pillar["vouch"]["keycloakClientSecret"] }}
  # You can find these values in Keycloak's OIDC info endpoint:
  # https://id.ashhhleyyy.dev/realms/Main/.well-known/openid-configuration
  auth_url: https://id.ashhhleyyy.dev/realms/Main/protocol/openid-connect/auth
  token_url: https://id.ashhhleyyy.dev/realms/Main/protocol/openid-connect/token
  user_info_url: https://id.ashhhleyyy.dev/realms/Main/protocol/openid-connect/userinfo
  scopes:
    - openid
  callback_url: https://vouch.ashhhleyyy.dev/auth
```

The `{{ pillar... }}` parts are [templating instructions](https://docs.saltproject.io/salt/user-guide/en/latest/topics/jinja.html) that fetch the secrets from my [Salt pillars](https://docs.saltproject.io/salt/user-guide/en/latest/topics/pillar.html). I then instruct Salt to run a container with the following configuration:

```yml
# Copy configuration file
vouch_config:
  file.managed:
    - name: /etc/vouch/config.yml
    - source: salt://etc/vouch/config.yml
    - template: jinja
    - makedirs: True

# Start container
vouch:
  docker_container.running:
    - name: vouch-main
    # Use voucher/vouch-proxy:latest if you aren't on an arm machine
    - image: voucher/vouch-proxy:latest-arm
    - restart_policy: on-failure:5
    # Port 9090 is already taken by another service for me
    - port_bindings:
      - 9099:9090
    # Mount the configuration file into the container
    - binds:
      - /etc/vouch:/config
```

Once I had Vouch running, the next step was to proxy it with nginx:

```
server {
	# Standard HTTPS stuff
	listen 443 ssl;
	listen [::]:443 ssl;
	
	ssl_certificate /etc/letsencrypt/live/ashhhleyyy.dev/fullchain.pem;
	ssl_certificate_key /etc/letsencrypt/live/ashhhleyyy.dev/privkey.pem;

	server_name vouch.ashhhleyyy.dev;

	location / {
		proxy_pass http://localhost:9099;
		proxy_set_header Host $http_host;
	}
}
```

Finally, I can update the port-exposing nginx config to authenticate requests through Vouch:

```
map $http_upgrade $connection_upgrade {
	default upgrade;
	'' close;
}

server {
	listen 443 ssl;
	listen [::]:443 ssl;

	ssl_certificate /etc/letsencrypt/live/ashhhleyyy.dev/fullchain.pem;
	ssl_certificate_key /etc/letsencrypt/live/ashhhleyyy.dev/privkey.pem;

	server_name "~^(?<port>[0-9]{1,5})-internal\.ashhhleyyy\.dev$";

	# 👇 New: pass requests through the /validate endpoint to check authentication
	auth_request /validate;

	location / {
		proxy_pass http://100.111.252.38:$port;
		# 👇 New: pass the user to the service I case I want to use that in future
		proxy_set_header X-Vouch-User $auth_resp_x_vouch_user;
		proxy_http_version 1.1;
		proxy_set_header Upgrade $http_upgrade;
		proxy_set_header Connection $connection_upgrade;
		proxy_set_header Host $host;
	}

	# 👇 New
	location = /validate {
		proxy_pass http://127.0.0.1:9099/validate;
		proxy_set_header Host $http_host;
		proxy_pass_request_body off;
		proxy_set_header Content-Length "";
		auth_request_set $auth_resp_x_vouch_user $upstream_http_x_vouch_user;
		auth_request_set $auth_resp_jwt $upstream_http_x_vouch_jwt;
		auth_request_set $auth_resp_err $upstream_http_x_vouch_err;
		auth_request_set $auth_resp_failcount $upstream_http_x_vouch_failcount;
	}

	# 👇 New
	error_page 401 = @error401;

	# 👇 New
	location @error401 {
		# Redirect the request to Vouch to authenticate
		return 302 https://vouch.ashhhleyyy.dev/login?url=$scheme://$http_host$request_uri&vouch-failcount=$auth_resp_failcount&X-Vouch-Token=$auth_resp_jwt&error=$auth_resp_err;
	}
}
```

Now, when you try to access any of these domains, you'll be greeted by a login page:

![A login page, styled in the same way as this website](/assets/images/programming-at-school-2/keycloak-login.png)

...

Maybe I should write another post about setting up Keycloak and making my custom login theme 🤔...
