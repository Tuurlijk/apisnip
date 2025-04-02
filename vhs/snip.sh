#!/usr/bin/env bash

vhs vhs/apisnip.tape

cp target/apisnip.gif ../

git checkout images

mv ../apisnip.gif images/

git add .
git commit -m "Update apisnip.gif"
git push
