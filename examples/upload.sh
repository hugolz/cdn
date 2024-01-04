#!/bin/bash

username="hugo"
file_path="./assets/rocket.png"

file_ext="png"
endpoint="http://192.168.1.24:8000/json" 

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

# Data binary is used to send larger amount of data
curl "$endpoint" \
  --request POST \
  --header "Content-Type: application/json" \
  --data-binary '@-' << EOF
  $json_payload 
EOF
