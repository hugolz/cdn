#!/bin/bash

id="3d647372-cf66-412f-92ae-18c3dcc9ca72"

endpoint="http://127.0.0.1:8000/" 

curl "${endpoint}${id}" > out.json

python parse.py
