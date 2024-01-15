#!/bin/bash

id="76860cbd-37ce-4542-b115-d4fb60823e50"

endpoint="http://172.26.224.1:8001/download/" 

curl "${endpoint}${id}" > out.json

python parse.py
