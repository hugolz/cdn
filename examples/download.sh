#!/bin/bash

id="406a3f75-bf9a-4c82-89bb-b87bd877e34c"

endpoint="http://192.168.1.24:8001/" 

curl "${endpoint}${id}" > out.json

python parse.py
