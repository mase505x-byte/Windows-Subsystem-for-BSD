#!/bin/bash
groff -man -Tutf8 man.1 | less -R
g++ -std=c++20 -O2 shell.cpp -o wsb.exe