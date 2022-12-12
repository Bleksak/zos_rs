#!/bin/bash

#skript pro kontrolu obsahu input/ a output/ složek

for a in $(ls input) ; do
	if test -f "output/$a" ; then
    		cmp "input/$a" "output/$a" && echo "$a OK"
	else
		echo "soubor output/$a neexistuje!" 
	fi
done
