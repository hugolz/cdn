#!/bin/bash

id="cfaf675a-b929-4d82-ac10-d2e38814988a"

endpoint="http://192.168.1.24:8000/" 

curl "${endpoint}${id}" > out.json

python parse.py
