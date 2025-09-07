#!/usr/bin/env bash

curl 'https://jenkins.service.isnt-a.top/job/Buxus%20update%20app/buildWithParameters' \
  --user $JENKINS_USER:$JENKINS_TOKEN \
  --data APP_NAME=ashhhleyyy-dot-dev \
  --data IMAGE_NAME=ghcr.io/ashhhleyyy/website \
  --data IMAGE_TAG=$GITHUB_SHA
