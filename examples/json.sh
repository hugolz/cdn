#!/bin/bash

username="hugo"
file_path="assets/rocket.png"
file_ext="png"
endpoint="http://127.0.0.1:8000/json" 

# Don't forget the -w 0 else the output will be escaped and escape chars are not allowed in serde::json
file_base64=$(base64 -w 0 "$file_path")

json_payload=$(cat <<EOF
{
  "metadata": {
    "username": "$username",
    "file_ext": "$file_ext"
  },
  "file": "$file_base64"
}
EOF
)

curl --request POST --header "Content-Type: application/json" --data-ascii "$json_payload" "$endpoint"
