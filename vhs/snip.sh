#!/usr/bin/env bash

cargo install --path .

vhs vhs/apisnip.tape

cp target/apisnip.gif ../

git checkout images

mv ../apisnip.gif images/

git add .
git commit -m "Update apisnip.gif"
git push

git checkout main

xdg-open images/apisnip.gif