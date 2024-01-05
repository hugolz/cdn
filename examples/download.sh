#!/bin/bash

id="96acf314-531c-4fc8-89cc-c585e0ced6da"

endpoint="http://192.168.1.24:8000/" 

curl "${endpoint}${id}" > out.json

python parse.py
