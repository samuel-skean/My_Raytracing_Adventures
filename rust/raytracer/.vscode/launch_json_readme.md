God I hate that json doesn't support comments.

I tried using the launch.json file here to debug on macOS in VSCode using CodeLLDB, but every breakpoint I set turned grey as soon as I ran the file. They started out red and filled-in, but then they turned grey and hollow. Much more importantly, they didn't trigger. Is it multithreading that's the problem?

This launch.json file should be *very close* to the default. The only difference I know of is that I added arguments so it could actually run the binary I gave it, on line 22. That change seemed to work, at least.

I don't know, but I'm next gonna try rust-gdb on linux. I would try it on macOS, but I still really don't want to use brew or whatever, and I'm not quite ready to use the nix package manager yet.