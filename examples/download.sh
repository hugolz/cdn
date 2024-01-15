#!/bin/bash

id="53cf83fc-390d-4de9-9449-f8db3b239bc1"

endpoint="http://192.168.1.24:8001/download/" 

curl "${endpoint}${id}" > out.json

python parse.py
