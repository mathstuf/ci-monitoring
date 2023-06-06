#!/bin/bash

set -e
shopt -s extglob

readonly username="$1"
shift

token="$( eval "echo \${DOCKERHUB_TOKEN_${username}}" )"
readonly token

pr_token="$( curl --silent --user "$username:$token" "https://auth.docker.io/token?service=registry.docker.io&scope=repository:ratelimitpreview/test:pull" | jq -r .token )"
readonly pr_token

curl --silent --head -H "Authorization: Bearer $pr_token" "https://registry-1.docker.io/v2/ratelimitpreview/test/manifests/latest" | tr -d $'\r' > dockerhub-header

rate_source="$( grep -i 'docker-ratelimit-source:' dockerhub-header | cut -d' ' -f2 )"
readonly rate_source

limit="$( grep -i 'ratelimit-limit:' dockerhub-header | cut -d' ' -f2 )"
readonly limit

remaining="$( grep -i 'ratelimit-remaining:' dockerhub-header | cut -d' ' -f2 )"
readonly remaining

echo "User                : $username"
echo "Ratelimit source    : $rate_source"

print_rates () {
    local describe="$1"
    readonly describe
    shift

    local total="$1"
    readonly total
    shift

    local retvar="$1"
    readonly retvar

    local count
    local window

    local param
    IFS=";"
    for param in $total; do
        case "$param" in
            w=*)
                window="$( echo "$param" | cut -d= -f2 )"
                ;;
            [0-9]*)
                count="$param"
                ;;
        esac
    done

    readonly count
    readonly window

    echo "Rate limit $describe: $count ($window seconds)"

    if [ -n "$retvar" ]; then
        eval "$retvar=$count"
    fi
}

if [ -z "$limit" ] && [ -z "$remaining" ]; then
    echo "Rate limit:           none"
else
    print_rates "total    " "$limit"
    print_rates "remaining" "$remaining" remaining_count
    readonly remaining_count

    if [ "$remaining_count" -lt 500 ]; then
        exit 2
    elif [ "$remaining_count" -lt 100 ]; then
        exit 1
    fi
fi
