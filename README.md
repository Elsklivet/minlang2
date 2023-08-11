# minlang2
*The sequel.*

When I was in my second year of college, I wrote a little BrainF\*\*\*-like esoteric language interpreter in Python, because I was bored and it only took the time between a couple classes. 
Unfortunately, that implementation sucked. Not just because it was written in Python and liable to hit runtime errors at every corner, but also because it was extraordinarily naïve.
For example, it could not correctly parse if blocks unless they were at the end of function definitions, because both ended their blocks with '`;`', and it caught the first one and said that was the end of everything.

This repository is my attempt at recreating an actually working reference implementation of that same toy language, this time with a real compiler frontend to lex and parse it. My goal is to eventually give this a VM to run it and maybe a compiler backend into NASM, x86, or MIPS (or some compiler-compiler input?). For now, however, this is a work-in-progress as I try to get the language on its feet again. I'll update this as I add features. I also really, really need to put some doc comments around so this thing is legible, at all. Will get to that eventually...

Until I add a more in-depth view of the language, feel free to refer to the [original repository](https://github.com/Elsklivet/minlang), or to the [EBNF language definition](/language.ebnf).

Cheers.

~Elsklivet