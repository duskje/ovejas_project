#!/bin/bash

DOWNLOAD_URL=$(curl -s -L -H "Accept: application/vnd.github+json" -H "Authorization: Bearer $GITHUB_TOKEN" -H "X-GitHub-Api-Version: 2022-11-28" https://api.github.com/repos/duskje/ovejas_project/actions/artifacts | jq -r '.artifacts | sort_by(.created_at) | .[-1] | .archive_download_url')
echo $DOWNLOAD_URL
curl -s -L -H "Accept: application/vnd.github+json" -H "Authorization: Bearer $GITHUB_TOKEN" -H "X-GitHub-Api-Version: 2022-11-28" $DOWNLOAD_URL -o ovejas_webserver.zip
unzip ovejas_webserver.zip
